use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{self, request::Parts},
    RequestPartsExt,
};

pub use axum::extract::FromRef;
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use oxidauth_kernel::{base64::*, jwt::Jwt, public_keys::PublicKey};

use crate::OxidAuthClient;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractJwt(pub Jwt);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractJwt
where
    OxidAuthClient: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = http::StatusCode;

    #[tracing::instrument(
        name = "oxidauth extract jwt",
        level = "trace",
        skip_all,
        ret,
        err
    )]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        let client = OxidAuthClient::from_ref(state);

        let public_keys_result = client
            .list_all_public_keys()
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        for PublicKey { public_key, .. } in public_keys_result
            .public_keys
            .into_iter()
        {
            let decoded = match BASE64_STANDARD.decode(public_key) {
                Ok(decoded) => decoded,
                Err(_) => continue,
            };

            if let Ok(jwt) = Jwt::decode(bearer.token(), &decoded) {
                return Ok(ExtractJwt(jwt));
            }
        }

        Err(http::StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractEntitlements(pub Vec<String>);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractEntitlements
where
    OxidAuthClient: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = http::StatusCode;

    #[tracing::instrument(
        name = "oxidauth extract entitlements",
        level = "trace",
        skip_all,
        ret,
        err
    )]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let ExtractJwt(jwt) =
            ExtractJwt::from_request_parts(parts, state).await?;

        let entitlements = match jwt.entitlements {
            Some(entitlements) => entitlements,
            None => return Ok(ExtractEntitlements(Vec::new())),
        };

        let permissions: Vec<_> = entitlements
            .split(' ')
            .map(|s| s.to_owned())
            .collect();

        Ok(ExtractEntitlements(
            permissions,
        ))
    }
}
