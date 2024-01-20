use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::{RolePermission, RolePermissionGrant};

pub type DeleteRolePermissionGrantService = Arc<
    dyn for<'a> Service<
        &'a DeleteRolePermissionGrant,
        Response = RolePermission,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteRolePermissionGrant {
    pub role_id: Uuid,
    pub permission: String,
}

