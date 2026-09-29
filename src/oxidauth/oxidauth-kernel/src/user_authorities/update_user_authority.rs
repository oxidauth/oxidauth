use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub use crate::service::Service;
use crate::{error::BoxedError, JsonValue};

pub use super::UserAuthority;

pub type UpdateUserAuthorityService = Arc<
    dyn for<'a> Service<
        &'a UpdateUserAuthority,
        Response = UserAuthority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub params: JsonValue,
}
