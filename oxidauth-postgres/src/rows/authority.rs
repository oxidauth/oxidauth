use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct AuthorityRow {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: String,
    pub strategy: String,
    pub params: Value,
    pub settings: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct InsertAuthority {
    pub name: String,
    pub client_key: Uuid,
    pub status: String,
    pub strategy: String,
    pub params: Value,
    pub settings: Value,
}

pub struct UpdateAuthority {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: String,
    pub strategy: String,
    pub params: Value,
    pub settings: Value,
}
