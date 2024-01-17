use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::roles::Role;

pub mod list_role_role_grants_by_parent_id;
pub mod create_role_role_grant;
pub mod delete_role_role_grant;

#[derive(Debug, Serialize)]
pub struct RoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RoleRoleGrantDetail {
    pub role: Role,
    pub grant: RoleRoleGrant,
}
