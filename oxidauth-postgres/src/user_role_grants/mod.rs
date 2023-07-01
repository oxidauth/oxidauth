pub mod delete_user_role_grant;
pub mod insert_user_role_grant;

use oxidauth_repository::user_role_grants::UserRoleGrantRow as RepoUserRoleGrantRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct UserRoleGrantRow {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRoleGrantRow> for RepoUserRoleGrantRow {
    fn from(value: UserRoleGrantRow) -> Self {
        Self {
            user_id: value.user_id,
            role_id: value.role_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
