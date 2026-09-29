pub mod find_most_recent_private_key;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct PrivateKey {
    pub id: Uuid,
    pub private_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
