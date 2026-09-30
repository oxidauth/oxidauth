use oxidauth_kernel::permissions::create_permission::{CreatePermission, Permission};
use serde::{Deserialize, Serialize};

pub type CreatePermissionReq = CreatePermission;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePermissionRes {
    pub permission: Permission,
}
