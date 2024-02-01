pub mod delete_user_authority;
pub mod insert_user_authority;
pub mod select_user_authorities_by_authority_id_and_user_identifier;
pub mod select_user_authorities_by_user_id;
pub mod select_user_authority_by_user_id_and_authority_id;
pub mod update_user_authority;

use std::{fmt, str::FromStr};

use oxidauth_kernel::{
    authorities::{Authority, AuthorityStatus, AuthorityStrategy},
    user_authorities::{UserAuthority, UserAuthorityWithAuthority},
    JsonValue,
};

use crate::prelude::*;

#[derive(sqlx::FromRow)]
struct PgUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl fmt::Debug for PgUserAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgUserAuthority")
            .field("user_id", &self.user_id)
            .field(
                "authority_id",
                &self.authority_id,
            )
            .field(
                "user_identifier",
                &self.user_identifier,
            )
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl From<PgUserAuthority> for UserAuthority {
    fn from(value: PgUserAuthority) -> Self {
        Self {
            user_id: value.user_id,
            authority_id: value.authority_id,
            user_identifier: value.user_identifier,
            params: JsonValue(value.params),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
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

impl fmt::Debug for PgUserAuthorityWithAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgUserAuthorityWithAuthority")
            .field("user_id", &self.user_id)
            .field(
                "authority_id",
                &self.authority_id,
            )
            .field(
                "user_identifier",
                &self.user_identifier,
            )
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .field(
                "authority_name",
                &self.authority_name,
            )
            .field(
                "authority_client_key",
                &self.authority_client_key,
            )
            .field(
                "authority_status",
                &self.authority_status,
            )
            .field(
                "authority_strategy",
                &self.authority_strategy,
            )
            .field(
                "authority_settings",
                &self.authority_settings,
            )
            .field(
                "authority_status",
                &self.authority_status,
            )
            .field(
                "authority_created_at",
                &self.authority_created_at,
            )
            .field(
                "authority_updated_at",
                &self.authority_updated_at,
            )
            .finish()
    }
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
                params: JsonValue(value.params),
                created_at: value.created_at,
                updated_at: value.updated_at,
            },
            authority: Authority {
                id: value.authority_id,
                name: value.authority_name,
                client_key: value.authority_client_key,
                status: AuthorityStatus::from_str(&value.authority_status)?,
                strategy: AuthorityStrategy::from_str(
                    &value.authority_strategy,
                )?,
                settings: serde_json::from_value(value.authority_settings)?,
                params: JsonValue(value.authority_params),
                created_at: value.authority_created_at,
                updated_at: value.authority_updated_at,
            },
        };

        Ok(user_authority)
    }
}
