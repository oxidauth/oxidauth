pub mod delete_public_key_by_id;
pub mod insert_public_key;
pub mod query_all_public_keys;
pub mod query_public_key_by_id;

use oxidauth_repository::public_keys::PublicKeyRow as RepoPublicKeyRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PublicKeyRow {
    pub id: Uuid,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PublicKeyRow> for RepoPublicKeyRow {
    fn from(value: PublicKeyRow) -> Self {
        Self {
            id: value.id,
            public_key: value.public_key,
            private_key: value.private_key,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
