use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::UserAuthority;

pub type CreateUserAuthorityService = Arc<
    dyn for<'a> Service<
        &'a CreateUserAuthority,
        Response = UserAuthority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct CreateUserAuthority {
    pub user_id: Option<Uuid>,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
}
