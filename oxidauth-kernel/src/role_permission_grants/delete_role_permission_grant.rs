use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::RolePermissionGrant;
pub use super::RolePermissionGrantDetail;

pub type DeleteRolePermissionGrantService = Arc<
    dyn for<'a> Service<
        &'a DeleteRolePermissionGrant,
        Response = RolePermissionGrantDetail,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteRolePermissionGrant {
    pub role_id: Uuid,
    pub permission: String,
}

