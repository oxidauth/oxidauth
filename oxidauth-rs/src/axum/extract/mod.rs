pub use axum::extract::FromRef;
use axum::{
    extract::FromRequestParts,
    http::{
        self,
        request::Parts,
    },
};
use axum_extra::{
    TypedHeader,
    headers::{
        Authorization,
        authorization::Bearer,
    },
};
use oxidauth_http::server::api::v1::public_keys::list_all_public_keys::ListAllPublicKeysRes;
use oxidauth_kernel::jwt::Jwt;
use tracing::error;
use uuid::Uuid;

use crate::{
    OxidAuthClient,
    client::public_keys::list_all_public_keys::ListAllPublicKeysTrait,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractJwt(pub Jwt);

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
        let client = OxidAuthClient::from_ref(state);

        #[cfg(feature = "mock")]
        if let Some(jwt) = client.mock_jwt {
            return Ok(ExtractJwt(jwt));
        }

        decode_jwt(parts, client).await
    }
}

async fn decode_jwt(
    parts: &mut Parts,
    client: OxidAuthClient,
) -> Result<ExtractJwt, http::StatusCode> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|err| {
            error!(msg = "error getting authorization header", err = ?err);

            http::StatusCode::UNAUTHORIZED
        })?;

    let ListAllPublicKeysRes { public_keys } = client
        .list_all_public_keys()
        .await
        .map_err(|err| {
            error!(msg = "error getting public keys", err = ?err);

            http::StatusCode::UNAUTHORIZED
        })?;

    let jwt = Jwt::decode_with_public_keys(bearer.token(), &public_keys)
        .map_err(|err| {
            error!(msg = "error decoding public keys", err = ?err);

            http::StatusCode::UNAUTHORIZED
        })?;

    Ok(ExtractJwt(jwt))
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractEntitlements(pub Vec<String>);

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

        let permissions = jwt
            .entitlements
            .and_then(|entitlements| entitlements.as_vec())
            .unwrap_or_default();

        Ok(ExtractEntitlements(
            permissions,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractUserId(pub Uuid);

impl<S> FromRequestParts<S> for ExtractUserId
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
            error!("error getting sub from jwt");

            return Err(http::StatusCode::UNAUTHORIZED);
        };

        Ok(ExtractUserId(user_id))
    }
}
