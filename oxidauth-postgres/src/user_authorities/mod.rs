pub mod delete_user_authority;
pub mod delete_user_authority_by_id;
pub mod insert_user_authority;
pub mod query_user_authorities_by_user_id;
pub mod select_user_authorities_by_user_id;
pub mod select_user_authority_by_user_id_and_authority_id;
pub mod update_user_authority;

use oxidauth_kernel::{
    authorities::Authority,
    user_authorities::{UserAuthority, UserAuthorityWithAuthority},
};

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

#[derive(Debug, sqlx::FromRow)]
struct PgUserAuthorityWithAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authority_name: String,
    pub authority_client_key: Uuid,
    pub authority_status: String,
    pub authority_strategy: String,
    pub authority_settings: Value,
    pub authority_params: Value,
    pub authority_created_at: DateTime<Utc>,
    pub authority_updated_at: DateTime<Utc>,
}

impl TryFrom<PgUserAuthorityWithAuthority> for UserAuthorityWithAuthority {
    type Error = BoxedError;

    fn try_from(
        value: PgUserAuthorityWithAuthority,
    ) -> Result<Self, Self::Error> {
        let user_authority = Self {
            user_authority: UserAuthority {
                user_id: value.user_id,
                authority_id: value.authority_id,
                user_identifier: value.user_identifier,
                params: value.params,
                created_at: value.created_at,
                updated_at: value.updated_at,
            },
            authority: Authority {
                id: value.authority_id,
                name: value.authority_name,
                client_key: value.authority_client_key,
                status: value
                    .authority_status
                    .try_into()?,
                strategy: value
                    .authority_strategy
                    .try_into()?,
                settings: value.authority_settings,
                params: value.authority_params,
                created_at: value.authority_created_at,
                updated_at: value.authority_updated_at,
            },
        };

        Ok(user_authority)
    }
}
