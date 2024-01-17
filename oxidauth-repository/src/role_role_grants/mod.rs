pub mod delete_role_role_grant;
pub mod insert_role_role_grant;
pub mod select_role_role_grants_by_parent_id;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct RoleRoleGrantRow {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
