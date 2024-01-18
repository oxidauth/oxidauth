pub mod delete_user_authority;
pub mod delete_user_authority_by_id;
pub mod insert_user_authority;
pub mod query_user_authorities_by_user_id;
pub mod update_user_authority;

use oxidauth_kernel::user_authorities::UserAuthority;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
struct PgUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgUserAuthority> for UserAuthority {
    fn from(value: PgUserAuthority) -> Self {
        Self {
            user_id: value.user_id,
            authority_id: value.authority_id,
            user_identifier: value.user_identifier,
            params: value.params,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
