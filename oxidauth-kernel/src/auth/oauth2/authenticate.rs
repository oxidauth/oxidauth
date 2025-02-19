use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    JsonValue,
    dev_prelude::{BoxedError, Service},
};

pub type Oauth2AuthenticateService = Arc<
    dyn for<'a> Service<
            &'a Oauth2Authenticate,
            Response = Oauth2AuthenticateResponse,
            Error = BoxedError,
        >,
>;

#[derive(Debug, Clone, Deserialize)]
pub struct Oauth2Authenticate {
    pub code: String,
    pub scope: String,
    pub client_key: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Oauth2AuthenticateParams {
    pub code: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Oauth2AuthenticateResponse {
    pub profile: JsonValue,
}

#[derive(Debug)]
pub enum ParseOauth2AuthenticateUrlError {
    Unknown(String),
}

impl fmt::Display for ParseOauth2AuthenticateUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseOauth2AuthenticateUrlError::*;

        match self {
            Unknown(value) => write!(
                f,
                "unable to handle oauth authenticate response: {}",
                value
            ),
        }
    }
}
