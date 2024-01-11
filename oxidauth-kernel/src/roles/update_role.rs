use uuid::Uuid;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type UpdateRoleService = Arc<
    dyn for<'a> Service<
        &'a UpdateRole,
        Response = Role,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct UpdateRole {
    pub role_id: Option<Uuid>,
    pub name: String,
}
