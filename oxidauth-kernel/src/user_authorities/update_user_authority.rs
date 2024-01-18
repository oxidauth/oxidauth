use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::UserAuthority;

pub type UpdateUserAuthorityService = Arc<
    dyn for<'a> Service<
        &'a UpdateUserAuthority,
        Response = UserAuthority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct UpdateUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub params: Value,
}
