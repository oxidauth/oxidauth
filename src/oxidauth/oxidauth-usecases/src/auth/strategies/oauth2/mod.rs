pub mod authenticator;
pub mod google;
pub mod microsoft;
pub mod redirect;
pub mod registrar;
pub mod user_authority_from_request;
pub mod user_identifier_from_request;

use serde::{Deserialize, Serialize};
use url::Url;

use oxidauth_kernel::{JsonValue, error::BoxedError};
use uuid::Uuid;

#[derive(Debug)]
pub struct OAuth2 {
    authority_id: Uuid,
    pub params: AuthorityParams,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum OAuthFlavors {
    Google,
    Microsoft,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorityParams {
    pub exchange_url: Url, // Url where a token can be exchanged for an access code
    pub flavor: OAuthFlavors, // oauth2 platform
    pub oauth2_id: String, // client's sso id
    pub oauth2_secret: String, // client's sso secret
    pub profile_url: Url, // Url where an access code can be used to receive information from requested scopes
    pub scopes: String, // space separated list of scopes of user info requested from platform (might be a google only thing)
    pub redirect_url: Url, // Google Url where user goes through oauth flow
    pub client_base_url: Url, // Client frontend url that callback should redirect the user to after authenticating
    pub redirect_uri: Url, // Included in redirect url but used throughout the process as an extra security measure, required to match settings on oauth config
}

impl AuthorityParams {
    pub fn new(
        exchange_url: Url,
        flavor: OAuthFlavors,
        oauth2_id: String,
        oauth2_secret: String,
        profile_url: Url,
        scopes: String,
        redirect_url: Url,
        client_base_url: Url,
        redirect_uri: Url,
    ) -> Self {
        Self {
            exchange_url,
            flavor,
            oauth2_id,
            oauth2_secret,
            profile_url,
            scopes,
            redirect_url,
            client_base_url,
            redirect_uri,
        }
    }

    pub fn as_json_value(&self) -> Result<JsonValue, BoxedError> {
        Ok(JsonValue::new(serde_json::to_value(self)?))
    }
}

impl TryFrom<JsonValue> for AuthorityParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}
