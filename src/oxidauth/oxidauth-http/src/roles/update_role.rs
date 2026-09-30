use oxidauth_kernel::roles::update_role::{Role, UpdateRole};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UpdateRolePathReq {
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRoleReq {
    pub role: UpdateRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRoleRes {
    pub role: Role,
}
