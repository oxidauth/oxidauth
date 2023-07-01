pub mod delete_authority_by_id;
pub mod insert_authority;
pub mod query_all_authorities;
pub mod query_authority_by_client_id;
pub mod query_authority_by_id;
pub mod update_authority;

use oxidauth_repository::authorities::AuthorityRow as RepoAuthorityRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
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

impl From<AuthorityRow> for RepoAuthorityRow {
    fn from(value: AuthorityRow) -> Self {
        Self {
            id: value.id,
            name: value.name,
            client_key: value.client_key,
            status: value.status,
            strategy: value.strategy,
            settings: value.settings,
            params: value.params,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
