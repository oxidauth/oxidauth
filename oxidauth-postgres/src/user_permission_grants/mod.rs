pub mod delete_user_permission_grant;
pub mod insert_user_permission_grant;

use oxidauth_repository::user_permission_grants::UserPermissionGrantRow as RepoUserPermissionGrantRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct UserPermissionGrantRow {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserPermissionGrantRow> for RepoUserPermissionGrantRow {
    fn from(value: UserPermissionGrantRow) -> Self {
        Self {
            user_id: value.user_id,
            permission_id: value.permission_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
