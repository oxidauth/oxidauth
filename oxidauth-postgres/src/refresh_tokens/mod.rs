pub mod delete_refresh_token_by_expires_at;
pub mod insert_refresh_token;
pub mod select_refresh_token_by_id;

use oxidauth_kernel::refresh_tokens::RefreshToken;
use oxidauth_repository::refresh_tokens::RefreshTokenRow as RepoRefreshTokenRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgRefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgRefreshToken> for RepoRefreshTokenRow {
    fn from(value: PgRefreshToken) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            authority_id: value.authority_id,
            expires_at: value.expires_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<PgRefreshToken> for RefreshToken {
    fn from(value: PgRefreshToken) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            authority_id: value.authority_id,
            expires_at: value.expires_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
