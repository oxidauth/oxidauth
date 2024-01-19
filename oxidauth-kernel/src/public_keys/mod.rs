use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

pub mod find_public_key_by_id;
pub mod list_all_public_keys;

#[derive(Debug, Serialize)]
pub struct PublicKey {
    pub id: Uuid,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
