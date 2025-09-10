pub mod exchange_token;
pub mod retrieve_profile;

pub use exchange_token::exchange_microsoft_token;
pub use retrieve_profile::retrieve_microsoft_profile;

use serde::{Deserialize, Serialize};

use oxidauth_kernel::{JsonValue, error::BoxedError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftExchangeTokenReq<'a> {
    pub client_id: &'a str,
    pub scope: &'a str,
    pub code: &'a str,
    pub redirect_uri: &'a str,
    pub client_secret: &'a str,
    pub grant_type: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftExchangeTokenRes {
    pub token_type: String,
    pub scope: String,
    pub expires_in: u32,
    pub ext_expires_in: u32,
    pub access_token: String,
    pub id_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftProfile {
    pub display_name: String,
    pub given_name: String,
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
