pub mod delete_role_role_grant;
pub mod insert_role_role_grant;
pub mod select_role_role_grants_by_parent_id;

use oxidauth_kernel::{
    role_role_grants::{RoleRoleGrant, RoleRoleGrantDetail},
    roles::Role,
};
use oxidauth_repository::role_role_grants::RoleRoleGrantRow as RepoRoleRoleGrantRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgRoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgRoleRoleGrant> for RoleRoleGrant {
    fn from(val: PgRoleRoleGrant) -> Self {
        RoleRoleGrant {
            parent_id: val.parent_id,
            child_id: val.child_id,
            created_at: val.created_at,
            updated_at: val.updated_at,
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

#[derive(Debug, sqlx::FromRow)]
pub struct PgRoleRoleGrantDetail {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role_name: String,
    pub role_created_at: DateTime<Utc>,
    pub role_updated_at: DateTime<Utc>,
}

impl From<PgRoleRoleGrantDetail> for RoleRoleGrantDetail {
    fn from(pg: PgRoleRoleGrantDetail) -> Self {
        Self {
            role: Role {
                id: pg.child_id,
                name: pg.role_name,
                created_at: pg.role_created_at,
                updated_at: pg.role_updated_at,
            },
            grant: RoleRoleGrant {
                parent_id: pg.parent_id,
                child_id: pg.child_id,
                created_at: pg.created_at,
                updated_at: pg.updated_at,
            },
        }
    }
}
