pub mod exchange_token;
pub mod retrieve_profile;

pub use exchange_token::exchange_google_token;
pub use retrieve_profile::retrieve_google_profile;

use serde::{Deserialize, Serialize};

use oxidauth_kernel::{JsonValue, error::BoxedError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleExchangeTokenReq {
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub grant_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleExchangeTokenRes {
    pub access_token: String,
    pub expires_in: u32,
    pub scope: String,
    pub token_type: String,
    pub id_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleProfile {
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub id: String,
    pub email: String,
    pub verified_email: bool,
}

impl TryFrom<JsonValue> for GoogleProfile {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let profile = serde_json::from_value(value.inner_value())?;

        Ok(profile)
    }
}
