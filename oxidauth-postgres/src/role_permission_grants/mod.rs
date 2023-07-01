pub mod delete_role_permission_grant;
pub mod insert_role_permission_grant;

use oxidauth_repository::role_permission_grants::RolePermissionGrantRow as RepoRolePermissionGrantRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct RolePermissionGrantRow {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<RolePermissionGrantRow> for RepoRolePermissionGrantRow {
    fn from(value: RolePermissionGrantRow) -> Self {
        Self {
            role_id: value.role_id,
            permission_id: value.permission_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
