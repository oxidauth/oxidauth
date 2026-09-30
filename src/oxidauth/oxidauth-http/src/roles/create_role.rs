use oxidauth_kernel::roles::create_role::{CreateRole, Role};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleReq {
    pub role: CreateRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRes {
    pub role: Role,
}
