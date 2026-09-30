use oxidauth_kernel::roles::list_all_roles::{ListAllRoles, Role};
use serde::{Deserialize, Serialize};

pub type ListAllRolesReq = ListAllRoles;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllRolesRes {
    pub roles: Vec<Role>,
}
