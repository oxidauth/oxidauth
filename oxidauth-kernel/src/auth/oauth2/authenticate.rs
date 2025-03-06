use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    JsonValue,
    dev_prelude::{BoxedError, Service},
};

pub type OAuth2AuthenticateService = Arc<
    dyn for<'a> Service<
            &'a OAuth2AuthenticateParams,
            Response = OAuth2AuthenticateResponse,
            Error = BoxedError,
        >,
>;

#[derive(Debug, Clone, Deserialize)]
pub struct OAuth2AuthenticateParams {
    pub code: String,
    pub scope: String,
    pub client_key: Uuid,
}

impl TryFrom<JsonValue> for OAuth2AuthenticateParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuth2AuthenticatePathParams {
    pub code: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuth2AuthenticateResponse {
    pub profile: JsonValue,
}

#[derive(Debug)]
pub enum ParseOAuth2AuthenticateUrlError {
    Unknown(String),
}

impl fmt::Display for ParseOAuth2AuthenticateUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseOAuth2AuthenticateUrlError::*;

        match self {
            Unknown(value) => write!(f, "unable to handle oauth authenticate response: {}", value),
        }
    }
}
