pub mod delete_authority;
pub mod insert_authority;
pub mod select_all_authorities;
pub mod query_authority_by_client_id;
pub mod select_authority_by_id;
pub mod update_authority;

use crate::prelude::*;

#[derive(Debug)]
pub struct AuthorityRow {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: String,
    pub strategy: String,
    pub settings: serde_json::Value,
    pub params: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
