use core::fmt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

use crate::{
    JsonValue,
    dev_prelude::{BoxedError, Service},
};

use super::{authenticate::AuthenticateParams, register::RegisterParams};

pub type AuthenticateOrRegisterService = Arc<
    dyn for<'a> Service<
            &'a AuthenticateOrRegisterParams,
            Response = AuthenticateOrRegisterResponse,
            Error = BoxedError,
        >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateOrRegisterResponse {
    pub jwt: String,
    pub refresh_token: Uuid,
    pub client_base: Url,
    pub email: String,
    pub given_name: String,
    pub family_name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuth2AuthenticateParams {
    pub code: String,
    pub scope: String,
    pub client_key: Uuid,
}

impl TryFrom<JsonValue> for OAuth2AuthenticateParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        serde_json::from_value(value.inner_value())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateOrRegisterParams {
    pub client_key: Uuid,
    pub params: JsonValue,
}

impl From<&AuthenticateOrRegisterParams> for AuthenticateParams {
    fn from(value: &AuthenticateOrRegisterParams) -> Self {
        Self {
            client_key: value.client_key,
            params: value.params.clone(),
        }
    }
}

impl From<&AuthenticateOrRegisterParams> for RegisterParams {
    fn from(value: &AuthenticateOrRegisterParams) -> Self {
        Self {
            client_key: value.client_key,
            params: value.params.clone(),
        }
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
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}
