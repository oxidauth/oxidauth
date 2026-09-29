pub mod delete_user_role_grant;
pub mod insert_user_role_grant;
pub mod select_user_role_grants_by_user_id;

use oxidauth_kernel::{
    roles::Role,
    user_role_grants::{UserRole, UserRoleGrant},
};

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgUserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgUserRoleGrant> for UserRoleGrant {
    fn from(value: PgUserRoleGrant) -> Self {
        Self {
            user_id: value.user_id,
            role_id: value.role_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PgUserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub role_created_at: DateTime<Utc>,
    pub role_updated_at: DateTime<Utc>,
}

impl From<PgUserRole> for UserRole {
    fn from(value: PgUserRole) -> Self {
        Self {
            role: Role {
                id: value.role_id,
                name: value.name,
                created_at: value.created_at,
                updated_at: value.updated_at,
            },
            grant: UserRoleGrant {
                user_id: value.user_id,
                role_id: value.role_id,
                created_at: value.created_at,
                updated_at: value.updated_at,
            },
        }
    }
}
