use uuid::Uuid;
use serde::Serialize;

pub mod create_role;
pub mod find_role_by_id;

#[derive(Debug, Serialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
}
