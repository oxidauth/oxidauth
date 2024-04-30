pub mod delete_role_permission_grant;
pub mod insert_role_permission_grant;
pub mod select_role_permission_grants_by_role_id;

use oxidauth_kernel::{
    permissions::Permission,
    role_permission_grants::{RolePermission, RolePermissionGrant},
};

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
struct PgRolePermissionGrant {
    role_id: Uuid,
    permission_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
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
pub struct PgRolePermission {
    role_id: Uuid,
    permission_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    realm: String,
    resource: String,
    action: String,
    permission_created_at: DateTime<Utc>,
    permission_updated_at: DateTime<Utc>,
}

impl From<PgRolePermission> for RolePermission {
    fn from(pg: PgRolePermission) -> Self {
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
