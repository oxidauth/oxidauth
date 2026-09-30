use oxidauth_kernel::roles::find_role_by_id::{FindRoleById, Role};
use serde::{Deserialize, Serialize};

pub type FindRoleByIdReq = FindRoleById;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRoleByIdRes {
    pub role: Role,
}
