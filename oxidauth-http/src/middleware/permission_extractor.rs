use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{self, request::Parts},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use oxidauth_kernel::{
    jwt::Jwt,
    public_keys::list_all_public_keys::{
        ListAllPublicKeys, ListAllPublicKeysService,
    },
};

use crate::provider::Provider;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractJwt(pub Jwt);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractJwt
where
    Provider: FromRef<S>,
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

        let provider = Provider::from_ref(state);

        let list_all_public_keys_service =
            provider.fetch::<ListAllPublicKeysService>();

        let public_keys = list_all_public_keys_service
            .call(&ListAllPublicKeys)
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        let jwt = Jwt::decode_with_public_keys(bearer.token(), &public_keys)
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        Ok(ExtractJwt(jwt))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractEntitlements(pub Vec<String>);

#[async_trait]
impl FromRequestParts<Provider> for ExtractEntitlements {
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
        state: &Provider,
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
