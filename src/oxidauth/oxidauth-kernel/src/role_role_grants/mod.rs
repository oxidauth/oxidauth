use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::roles::Role;

pub mod create_role_role_grant;
pub mod delete_role_role_grant;
pub mod list_role_role_grants_by_parent_id;

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleRoleGrantDetail {
    pub role: Role,
    pub grant: RoleRoleGrant,
}
