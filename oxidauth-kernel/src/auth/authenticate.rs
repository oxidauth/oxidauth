use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    JsonValue,
    authorities::find_authority_by_client_key::FindAuthorityByClientKey,
    dev_prelude::{BoxedError, Service},
};

pub type AuthenticateService = Arc<
    dyn for<'a> Service<&'a AuthenticateParams, Response = AuthenticateResponse, Error = BoxedError>,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticateParams {
    pub client_key: Uuid,
    pub params: JsonValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateResponse {
    pub jwt: String,
    pub refresh_token: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug)]
pub enum ParseAuthorityStrategyError {
    Unknown(String),
}

impl fmt::Display for ParseAuthorityStrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseAuthorityStrategyError::*;

        match self {
            Unknown(value) => write!(f, "unknown authority strategy: {}", value),
        }
    }
}

impl From<&AuthenticateParams> for FindAuthorityByClientKey {
    fn from(value: &AuthenticateParams) -> Self {
        Self {
            client_key: value.client_key,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookReq {
    pub webhook_key: String,
    pub name: Option<String>,
    pub email: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookRes {
    pub success: bool,
}
