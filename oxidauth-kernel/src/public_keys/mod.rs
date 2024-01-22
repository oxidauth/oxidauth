pub mod create_public_key;
pub mod delete_public_key;
pub mod find_public_key_by_id;
pub mod list_all_public_keys;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PublicKey {
    pub id: Uuid,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
