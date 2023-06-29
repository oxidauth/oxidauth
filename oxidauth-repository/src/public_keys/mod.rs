pub mod delete_public_key_by_id;
pub mod insert_public_key;
pub mod query_all_public_keys;
pub mod query_public_key_by_id;

use crate::prelude::*;

#[derive(Debug)]
pub struct PublicKeyRow {
    pub id: Uuid,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
