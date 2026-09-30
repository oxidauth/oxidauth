use oxidauth_kernel::permissions::delete_permission::{DeletePermission, Permission};
use serde::{Deserialize, Serialize};

pub type DeletePermissionReq = DeletePermission;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePermissionRes {
    pub permission: Permission,
}
