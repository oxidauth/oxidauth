pub mod consume_oauth_state;
pub mod insert_oauth_state;

use crate::Database;

#[derive(Debug, Clone)]
pub struct PgOAuthStateRepository {
    db: Database,
}

impl PgOAuthStateRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}
