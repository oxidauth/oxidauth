use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct PublicKeyRow {
    pub id: Uuid,
    pub public_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct InsertPublicKey {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}
