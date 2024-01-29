use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::{RolePermission, RolePermissionGrant};

pub type ListRolePermissionGrantsByRoleIdService = Arc<
    dyn for<'a> Service<
        &'a ListRolePermissionGrantsByRoleId,
        Response = Vec<RolePermission>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRolePermissionGrantsByRoleId {
    pub role_id: Uuid,
}
