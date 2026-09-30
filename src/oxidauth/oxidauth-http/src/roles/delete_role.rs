use oxidauth_kernel::roles::delete_role::{DeleteRole, Role};
use serde::{Deserialize, Serialize};

pub type DeleteRoleReq = DeleteRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRoleRes {
    pub role: Role,
}
