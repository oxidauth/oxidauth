use oxidauth_kernel::role_role_grants::delete_role_role_grant::{
    DeleteRoleRoleGrant,
    RoleRoleGrant,
};
use serde::{Deserialize, Serialize};

pub type DeleteRoleRoleGrantReq = DeleteRoleRoleGrant;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRoleRoleGrantRes {
    pub grant: RoleRoleGrant,
}
