use oxidauth_kernel::permissions::list_all_permissions::{ListAllPermissions, Permission};
use serde::{Deserialize, Serialize};

pub type ListAllPermissionsReq = ListAllPermissions;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllPermissionsRes {
    pub permissions: Vec<Permission>,
}
