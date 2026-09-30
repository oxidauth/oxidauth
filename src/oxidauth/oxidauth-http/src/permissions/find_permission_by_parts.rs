use oxidauth_kernel::permissions::find_permission_by_parts::{FindPermissionByParts, Permission};
use serde::{Deserialize, Serialize};

pub type FindPermissionByPartsReq = FindPermissionByParts;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPermissionByPartsRes {
    pub permission: Permission,
}
