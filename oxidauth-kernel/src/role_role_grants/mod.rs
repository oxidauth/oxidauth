use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;

pub mod list_all_role_role_grants;
pub mod create_role_role_grant;
pub mod delete_role_role_grant;

#[derive(Debug, Serialize)]
pub struct RoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
