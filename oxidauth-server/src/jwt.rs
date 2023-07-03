use std::{
    error::Error as StdError,
    ops::Add,
    time::{self, Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use axum::{
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http,
};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::server::routes::api::v1::{
    permissions::Permission,
    public_keys::all::{public_keys_all, PublicKeyRow},
};

pub type Error = Box<dyn StdError + Send + Sync + 'static>;

pub const DEFAULT_EXP_IN_SEC: u64 = 60 * 300;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jwt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<usize>,
    pub exp: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctx: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitlements: Option<String>,
}

impl Jwt {
    pub fn new() -> JwtBuilder {
        JwtBuilder::default()
    }
    pub fn encode(&self, key: &[u8]) -> Result<String, Error> {
        let key = EncodingKey::from_rsa_pem(key)?;
        let result = encode(&Header::new(Algorithm::RS256), self, &key)?;

        Ok(result)
    }

    pub fn decode(token: &str, key: &[u8]) -> Result<Jwt, Error> {
        let key = DecodingKey::from_rsa_pem(key)?;
        let result: TokenData<Jwt> = decode(&token, &key, &Validation::new(Algorithm::RS256))?;

        Ok(result.claims)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractJwt(pub Jwt);

#[async_trait]
impl<B> FromRequest<B> for ExtractJwt
where
    B: Send,
{
    type Rejection = http::StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(token)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        let Extension(db): Extension<PgPool> = Extension::from_request(req)
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        let mut db = db
            .acquire()
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        let private_keys = public_keys_all(&mut db)
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        for PublicKeyRow { public_key, .. } in private_keys.into_iter() {
            let decoded = base64::decode(public_key).map_err(|_| http::StatusCode::UNAUTHORIZED)?;

            if let Ok(jwt) = Jwt::decode(token.token(), &decoded) {
                return Ok(ExtractJwt(jwt));
            };
        }

        Err(http::StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractEntitlements(pub Vec<Permission>);

#[async_trait]
impl<B> FromRequest<B> for ExtractEntitlements
where
    B: Send,
{
    type Rejection = http::StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let ExtractJwt(jwt) = ExtractJwt::from_request(req)
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        let mut permissions = Vec::new();

        let entitlements = match jwt.entitlements {
            Some(entitlements) => entitlements,
            None => return Ok(ExtractEntitlements(permissions)),
        };

        for entitlement in entitlements.split(" ") {
            let permission = entitlement
                .try_into()
                .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

            permissions.push(permission);
        }

        permissions.sort();
        permissions.reverse();

        Ok(ExtractEntitlements(permissions))
    }
}

#[derive(Default)]
pub struct JwtBuilder {
    sub: Option<Uuid>,
    iss: Option<String>,
    aud: Option<String>,
    nbf: Option<Result<usize, Error>>,
    iat: Option<Result<usize, Error>>,
    ttl: Option<Duration>,
    exp: Option<usize>,
    ctx: Option<Value>,
    entitlements: Option<String>,
}

impl JwtBuilder {
    pub fn build(self) -> Result<Jwt, Error> {
        let JwtBuilder {
            sub,
            iss,
            aud,
            nbf,
            iat,
            ttl,
            exp,
            ctx,
            entitlements,
        } = self;

        let now = epoch_from_time(time::SystemTime::now())?;

        let nbf = Some(nbf.unwrap_or(Ok(now))?);

        let iat = Some(iat.unwrap_or(Ok(now))?);

        let exp_from_ttl = {
            let ttl = ttl.unwrap_or(Duration::from_secs(60 * 3));
            let exp = epoch_from_now(ttl)?;

            exp
        };

        let exp = exp.unwrap_or(exp_from_ttl);

        Ok(Jwt {
            sub,
            iss,
            aud,
            nbf,
            iat,
            exp,
            ctx,
            entitlements,
        })
    }

    pub fn with_subject(mut self, sub: Uuid) -> Self {
        self.sub = Some(sub);

        self
    }

    pub fn with_issuer(mut self, iss: String) -> Self {
        self.iss = Some(iss);

        self
    }

    pub fn with_audience(mut self, aud: String) -> Self {
        self.aud = Some(aud);

        self
    }

    pub fn with_not_before_from(mut self, duration: time::Duration) -> Self {
        let nbf = epoch_from_now(duration);

        self.nbf = Some(nbf);

        self
    }

    pub fn with_issued_at(mut self, issued_at: time::SystemTime) -> Self {
        let iat = epoch_from_time(issued_at);

        self.iat = Some(iat);

        self
    }

    pub fn with_expires_at(mut self, exp: usize) -> Self {
        self.exp = Some(exp);

        self
    }

    pub fn with_expires_in(mut self, duration: Duration) -> Self {
        self.ttl = Some(duration);

        self
    }

    pub fn with_entitlements(mut self, entitlements: Vec<String>) -> Self {
        self.entitlements = Some(entitlements.join(" "));

        self
    }
}

pub fn epoch_from_now(duration: Duration) -> Result<usize, Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .add(duration)
        .as_secs() as usize;

    Ok(expiration)
}

pub fn epoch_from_time(t: time::SystemTime) -> Result<usize, Error> {
    let epoch = t.duration_since(UNIX_EPOCH)?.as_secs() as usize;

    Ok(epoch)
}

#[cfg(test)]
mod tests {
    use crate::rsa::{generate, KeyPair};

    use super::*;

    #[test]
    fn works_with_rsa() {
        let KeyPair { public, private } = generate().unwrap();

        let claims = Jwt::new()
            .with_entitlements(vec!["realm:resource:action".to_string()])
            .with_expires_in(Duration::from_secs(60 * 300))
            .build()
            .unwrap();

        let encoded = claims.encode(&private).unwrap();

        let decoded = Jwt::decode(&encoded, &public).unwrap();

        assert_eq!(
            decoded.entitlements,
            Some("realm:resource:action".to_string())
        )
    }
}
