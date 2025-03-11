use async_trait::async_trait;
use serde::Deserialize;

use oxidauth_kernel::{
    JsonValue,
    auth::Authenticator,
    authorities::{Authority, UserAuthority},
    error::BoxedError,
};

use super::{AuthorityParams, OAuth2};

#[derive(Clone, Deserialize)]
pub struct AuthenticateParams {
    pub email: String,
    pub access_token: String,
}

impl TryFrom<JsonValue> for AuthenticateParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}

#[async_trait]
impl Authenticator for OAuth2 {
    #[tracing::instrument(name = "oauth2 authenticate", skip(self))]
    async fn authenticate(
        &self,
        _authenticate_params: JsonValue,
        authority: &Authority,
        user_authority: &UserAuthority,
    ) -> Result<(), BoxedError> {
        Ok(())
    }
}

#[tracing::instrument(name = "new oauth2 authenticator")]
pub async fn new(authority: &Authority) -> Result<Box<dyn Authenticator>, BoxedError> {
    let params: AuthorityParams = authority
        .params
        .clone()
        .try_into()?;

    Ok(Box::new(OAuth2 {
        authority_id: authority.id,
        params,
    }))
}
