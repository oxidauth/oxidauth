pub mod tree;

use crate::Database;

#[derive(Debug, Clone)]
pub struct PgAuthRepository {
    db: Database,
}

impl PgAuthRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}
