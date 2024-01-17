use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::Serialize;

pub mod create_role;
pub mod delete_role;
pub mod find_role_by_id;
pub mod list_all_roles;
pub mod update_role;

#[derive(Debug, Serialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
