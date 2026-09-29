pub mod delete_role;
pub mod insert_role;
pub mod select_all_roles;
pub mod select_role_by_id;
pub mod select_role_by_name;
pub mod update_role;

use oxidauth_kernel::roles::Role;
use oxidauth_repository::roles::RoleRow as RepoRoleRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgRole {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgRole> for Role {
    fn from(val: PgRole) -> Self {
        Role {
            id: val.id,
            name: val.name,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}

impl From<PgRole> for RepoRoleRow {
    fn from(value: PgRole) -> Self {
        Self {
            id: value.id,
            name: value.name,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
