use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::{
    ListRolePermissionGrantsByRoleId,
    RolePermission,
};
use serde::{Deserialize, Serialize};

pub type ListRolePermissionGrantsByRoleIdReq = ListRolePermissionGrantsByRoleId;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRolePermissionGrantsByRoleIdRes {
    pub permissions: Vec<RolePermission>,
}
