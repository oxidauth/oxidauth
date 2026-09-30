use oxidauth_kernel::role_role_grants::list_role_role_grants_by_parent_id::{
    ListRoleRoleGrantsByParentId,
    RoleRoleGrantDetail,
};
use serde::{Deserialize, Serialize};

pub type ListRoleRoleGrantsByParentIdReq = ListRoleRoleGrantsByParentId;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRoleRoleGrantsByParentIdRes {
    pub roles: Vec<RoleRoleGrantDetail>,
}
