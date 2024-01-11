use uuid::Uuid;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type DeleteRoleService = Arc<
    dyn for<'a> Service<
        &'a DeleteRole,
        Response = Role,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteRole {
    pub role_id: Uuid,
}
