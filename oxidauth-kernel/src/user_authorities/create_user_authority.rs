use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

pub use crate::service::Service;
use crate::{
    authorities::{
        find_authority_by_strategy::FindAuthorityByStrategy, AuthorityStrategy,
    },
    error::BoxedError,
};

pub use super::UserAuthority;

pub type CreateUserAuthorityService = Arc<
    dyn for<'a> Service<
        &'a CreateUserAuthorityParams,
        Response = UserAuthority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserAuthorityParams {
    pub user_id: Uuid,
    pub strategy: AuthorityStrategy,
    pub params: Value,
}

impl From<&CreateUserAuthorityParams> for FindAuthorityByStrategy {
    fn from(value: &CreateUserAuthorityParams) -> Self {
        Self {
            strategy: value.strategy,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserAuthority {
    pub user_id: Option<Uuid>,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
}
