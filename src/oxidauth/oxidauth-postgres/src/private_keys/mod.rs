use chrono::{DateTime, Utc};
use oxidauth_kernel::private_keys::PrivateKey;
use uuid::Uuid;

pub mod select_most_recent_private_key;

use crate::Database;

#[derive(Debug, Clone)]
pub struct PgPrivateKeyRepository {
    db: Database,
}

impl PgPrivateKeyRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PgPrivateKey {
    pub id: Uuid,
    pub private_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgPrivateKey> for PrivateKey {
    fn from(pg: PgPrivateKey) -> Self {
        Self {
            id: pg.id,
            private_key: pg.private_key,
            created_at: pg.created_at,
            updated_at: pg.updated_at,
        }
    }
}
