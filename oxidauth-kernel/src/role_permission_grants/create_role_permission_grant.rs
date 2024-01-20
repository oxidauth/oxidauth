use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::{RolePermission, RolePermissionGrant};

pub type CreateRolePermissionGrantService = Arc<
    dyn for<'a> Service<
        &'a CreateRolePermissionGrant,
        Response = RolePermission,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct CreateRolePermissionGrant {
    pub role_id: Uuid,
    pub permission: String,
}
