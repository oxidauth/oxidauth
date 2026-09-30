use oxidauth_kernel::{
    role_role_grants::create_role_role_grant::{CreateRoleRoleGrant, RoleRoleGrant},
    roles::Role,
};
use serde::{Deserialize, Serialize};

pub type CreateRoleRoleGrantReq = CreateRoleRoleGrant;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRoleGrantRes {
    pub child: Role,
    pub grant: RoleRoleGrant,
}
