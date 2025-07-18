pub mod exchange_token;
pub mod retrieve_profile;

pub use exchange_token::exchange_microsoft_token;
pub use retrieve_profile::retrieve_microsoft_profile;

use serde::{Deserialize, Serialize};

use oxidauth_kernel::{JsonValue, error::BoxedError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftExchangeTokenReq {
    pub client_id: String,
    pub scope: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_secret: String,
    pub grant_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftExchangeTokenRes {
    pub token_type: String,
    pub scope: String,
    pub expires_in: u32,
    pub ext_expires_in: u32,
    pub access_token: String,
    pub id_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftProfile {
    pub displayName: String,
    pub givenName: String,
    pub surname: String,
    pub id: String,
    pub mail: String,
}

impl TryFrom<JsonValue> for MicrosoftProfile {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let profile = serde_json::from_value(value.inner_value())?;

        Ok(profile)
    }
}
