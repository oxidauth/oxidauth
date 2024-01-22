use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    authorities::{
        find_authority_by_strategy::FindAuthorityByStrategy, AuthorityStrategy,
    },
    dev_prelude::{BoxedError, Service},
};

pub type RegisterService = Arc<
    dyn for<'a> Service<
        &'a RegisterParams,
        Response = RegisterResponse,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    pub strategy: AuthorityStrategy,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub jwt: String,
    pub refresh_token: Uuid,
}

impl From<&RegisterParams> for FindAuthorityByStrategy {
    fn from(value: &RegisterParams) -> Self {
        Self {
            strategy: value.strategy,
        }
    }
}
