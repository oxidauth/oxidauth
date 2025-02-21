pub mod authenticator;
pub mod redirect;

use serde::{Deserialize, Serialize};
use url::Url;

use oxidauth_kernel::{JsonValue, error::BoxedError};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum OauthProviders {
    Google,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorityParams {
    pub exchange_url: Url, // Url where a token can be exchanged for an access code
    pub flavor: OauthProviders, // oauth2 platform
    pub oauth2_id: String, // client's sso id
    pub oauth2_secret: String, // client's sso secret
    pub profile_url: Url, // Url where an access code can be used to receive information from requested scopes
    pub scopes: String, // space separated list of scopes of user info requested from platform (might be a google only thing)
    pub redirect_url: Url, // Google Url where user goes through oauth flow
}

impl AuthorityParams {
    pub fn new(
        exchange_url: Url,
        flavor: OauthProviders,
        oauth2_id: String,
        oauth2_secret: String,
        profile_url: Url,
        scopes: String,
        redirect_url: Url,
    ) -> Self {
        Self {
            exchange_url,
            flavor,
            oauth2_id,
            oauth2_secret,
            profile_url,
            scopes,
            redirect_url,
        }
    }

    pub fn as_json_value(&self) -> Result<JsonValue, BoxedError> {
        Ok(JsonValue::new(
            serde_json::to_value(self)?,
        ))
    }
}

impl TryFrom<JsonValue> for AuthorityParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}
