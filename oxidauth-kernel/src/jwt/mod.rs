use std::ops::Add;
use std::time::{self, Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};

use crate::dev_prelude::*;

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

    pub fn encode(&self, key: &[u8]) -> Result<String, JwtError> {
        let key = EncodingKey::from_rsa_pem(key).map_err(|_| JwtError {})?;

        let result = encode(
            &Header::new(Algorithm::RS256),
            self,
            &key,
        )
        .map_err(|_| JwtError {})?;

        Ok(result)
    }

    pub fn decode(token: &str, key: &[u8]) -> Result<Jwt, JwtError> {
        let key = DecodingKey::from_rsa_pem(key).map_err(|_| JwtError {})?;

        let result: TokenData<Jwt> = decode(
            token,
            &key,
            &Validation::new(Algorithm::RS256),
        )
        .map_err(|_| JwtError {})?;

        Ok(result.claims)
    }
}

#[derive(Debug)]
pub struct JwtError {}

#[derive(Default)]
pub struct JwtBuilder {
    sub: Option<Uuid>,
    iss: Option<String>,
    aud: Option<String>,
    nbf: Option<Result<usize, JwtError>>,
    iat: Option<Result<usize, JwtError>>,
    ttl: Option<Duration>,
    exp: Option<usize>,
    ctx: Option<Value>,
    entitlements: Option<String>,
}

impl JwtBuilder {
    pub fn build(self) -> Result<Jwt, JwtError> {
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
            epoch_from_now(ttl)?
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

pub fn epoch_from_now(duration: Duration) -> Result<usize, JwtError> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| JwtError {})?
        .add(duration)
        .as_secs() as usize;

    Ok(expiration)
}

pub fn epoch_from_time(t: time::SystemTime) -> Result<usize, JwtError> {
    let epoch = t
        .duration_since(UNIX_EPOCH)
        .map_err(|_| JwtError {})?
        .as_secs() as usize;

    Ok(epoch)
}

// #[cfg(test)]
// mod tests {
//     use crate::rsa::{generate, KeyPair};
//
//     use super::*;
//
//     #[test]
//     fn works_with_rsa() {
//         let KeyPair { public, private } = generate().unwrap();
//
//         let claims = Jwt::new()
//             .with_entitlements(vec!["realm:resource:action".to_string()])
//             .with_expires_in(Duration::from_secs(60 * 300))
//             .build()
//             .unwrap();
//
//         let encoded = claims.encode(&private).unwrap();
//
//         let decoded = Jwt::decode(&encoded, &public).unwrap();
//
//         assert_eq!(
//             decoded.entitlements,
//             Some("realm:resource:action".to_string())
//         )
//     }
// }
