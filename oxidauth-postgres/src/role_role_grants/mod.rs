pub mod delete_role_role_grant;
pub mod insert_role_role_grant;

use oxidauth_repository::role_role_grants::RoleRoleGrantRow as RepoRoleRoleGrantRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct RoleRoleGrantRow {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<RoleRoleGrantRow> for RepoRoleRoleGrantRow {
    fn from(value: RoleRoleGrantRow) -> Self {
        Self {
            parent_id: value.parent_id,
            child_id: value.child_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
