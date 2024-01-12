pub mod delete_role_role_grant;
pub mod insert_role_role_grant;

use oxidauth_kernel::role_role_grants::RoleRoleGrant;
use oxidauth_repository::role_role_grants::RoleRoleGrantRow as RepoRoleRoleGrantRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgRoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Into<RoleRoleGrant> for PgRoleRoleGrant {
    fn into(self) -> RoleRoleGrant {
        RoleRoleGrant {
            parent_id: self.parent_id,
            child_id: self.child_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl From<PgRoleRoleGrant> for RepoRoleRoleGrantRow {
    fn from(value: PgRoleRoleGrant) -> Self {
        Self {
            parent_id: value.parent_id,
            child_id: value.child_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
