use uuid::Uuid;
use serde::Serialize;

pub mod create_role;

#[derive(Debug, Serialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
}
