use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    authorities::{
        find_authority_by_strategy::FindAuthorityByStrategy, AuthorityStrategy,
    },
    dev_prelude::{BoxedError, Service},
};

pub type AuthenticateService = Arc<
    dyn for<'a> Service<
        &'a AuthenticateParams,
        Response = AuthenticateResponse,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticateParams {
    pub strategy: AuthorityStrategy,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateResponse {
    pub jwt: String,
    pub refresh_token: Uuid,
}

#[derive(Debug)]
pub enum ParseAuthorityStrategyError {
    Unknown(String),
}

impl fmt::Display for ParseAuthorityStrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseAuthorityStrategyError::*;

        match self {
            Unknown(value) => write!(
                f,
                "unknown authority strategy: {}",
                value
            ),
        }
    }
}

impl From<&AuthenticateParams> for FindAuthorityByStrategy {
    fn from(value: &AuthenticateParams) -> Self {
        Self {
            strategy: value.strategy,
        }
    }
}
