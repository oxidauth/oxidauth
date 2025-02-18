pub mod authenticator;
pub mod redirect;
pub mod registrar;

use serde::{Deserialize, Serialize};
use url::Url;

use oxidauth_kernel::{JsonValue, error::BoxedError};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum OauthProviders {
    Google,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorityParams {
    redirect_url: Url,
    exchange_url: Url,
    oauth2_id: String,
    oauth2_secret: String,
    oauth2_flavor: OauthProviders,
}

impl AuthorityParams {
    pub fn new(
        redirect_url: Url,
        exchange_url: Url,
        oauth2_id: String,
        oauth2_secret: String,
        oauth2_flavor: OauthProviders,
    ) -> Self {
        Self {
            redirect_url,
            exchange_url,
            oauth2_id,
            oauth2_secret,
            oauth2_flavor,
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
