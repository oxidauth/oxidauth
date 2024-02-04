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
use oxidauth_http::server::api::v1::public_keys::list_all_public_keys::ListAllPublicKeysRes;
use oxidauth_kernel::jwt::Jwt;
use uuid::Uuid;

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

        let ListAllPublicKeysRes { public_keys } = client
            .list_all_public_keys()
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        let jwt = Jwt::decode_public_keys(bearer.token(), &public_keys)
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        Ok(ExtractJwt(jwt))
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserId(Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for UserId
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

        let Some(user_id) = jwt.sub else {
            return Err(http::StatusCode::UNAUTHORIZED);
        };

        Ok(UserId(user_id))
    }
}
