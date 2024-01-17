pub mod delete_role_permission_grant;
pub mod insert_role_permission_grant;
pub mod select_role_permission_grants_by_role_id;

use oxidauth_kernel::{
    permissions::Permission,
    role_permission_grants::{RolePermissionGrant, RolePermissionGrantDetail},
};

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgRolePermissionGrant {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgRolePermissionGrant> for RolePermissionGrant {
    fn from(pg: PgRolePermissionGrant) -> Self {
        Self {
            role_id: pg.role_id,
            permission_id: pg.permission_id,
            created_at: pg.created_at,
            updated_at: pg.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PgRolePermissionGrantDetail {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub permission_created_at: DateTime<Utc>,
    pub permission_updated_at: DateTime<Utc>,
}

impl From<PgRolePermissionGrantDetail> for RolePermissionGrantDetail {
    fn from(pg: PgRolePermissionGrantDetail) -> Self {
        Self {
            permission: Permission {
                id: pg.permission_id,
                realm: pg.realm,
                resource: pg.resource,
                action: pg.action,
                created_at: pg.permission_created_at,
                updated_at: pg.permission_updated_at,
            },
            grant: RolePermissionGrant {
                role_id: pg.role_id,
                permission_id: pg.permission_id,
                created_at: pg.created_at,
                updated_at: pg.updated_at,
            },
        }
    }
}
