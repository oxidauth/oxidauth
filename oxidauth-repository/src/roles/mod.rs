pub mod delete_role_by_id;
pub mod insert_role;
pub mod select_all_roles;
pub mod select_role_by_id;
pub mod update_role;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct RoleRow {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
