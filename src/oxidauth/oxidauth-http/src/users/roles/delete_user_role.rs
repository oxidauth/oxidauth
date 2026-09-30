use oxidauth_kernel::user_role_grants::delete_user_role_grant::{DeleteUserRoleGrant, UserRole};
use serde::{Deserialize, Serialize};

pub type DeleteUserRoleReq = DeleteUserRoleGrant;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserRoleRes {
    pub user_role: UserRole,
}
