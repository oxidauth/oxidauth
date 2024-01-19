use std::str::FromStr;

pub mod delete_authority;
pub mod insert_authority;
pub mod query_authority_by_client_id;
pub mod select_all_authorities;
pub mod select_authority_by_strategy;
pub mod select_authority_by_id;
pub mod update_authority;

use oxidauth_kernel::authorities::{
    Authority, AuthorityStatus, AuthorityStrategy,
};
use oxidauth_repository::authorities::AuthorityRow as RepoAuthorityRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
struct PgAuthority {
    id: Uuid,
    name: String,
    client_key: Uuid,
    status: String,
    strategy: String,
    settings: serde_json::Value,
    params: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PgAuthority> for RepoAuthorityRow {
    fn from(value: PgAuthority) -> Self {
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

impl TryFrom<PgAuthority> for Authority {
    type Error = BoxedError;

    fn try_from(value: PgAuthority) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            name: value.name,
            client_key: value.client_key,
            status: AuthorityStatus::from_str(&value.status)?,
            strategy: AuthorityStrategy::from_str(&value.strategy)?,
            settings: serde_json::from_value(value.settings)?,
            params: value.params,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}
