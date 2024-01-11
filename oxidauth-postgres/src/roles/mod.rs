pub mod delete_role_by_id;
pub mod insert_role;
pub mod select_role_by_id;
pub mod update_role;

use oxidauth_kernel::roles::Role;
use oxidauth_repository::roles::RoleRow as RepoRoleRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct RoleRow {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Into<Role> for RoleRow {
    fn into(self) -> Role {
        Role {
            id: self.id,
            name: self.name,
        }
    }
}

impl From<RoleRow> for RepoRoleRow {
    fn from(value: RoleRow) -> Self {
        Self {
            id: value.id,
            name: value.name,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
