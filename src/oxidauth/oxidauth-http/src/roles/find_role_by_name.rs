use oxidauth_kernel::roles::find_role_by_name::{FindRoleByName, Role};
use serde::{Deserialize, Serialize};

pub type FindRoleByNameReq = FindRoleByName;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRoleByNameRes {
    pub role: Role,
}
