pub mod delete_role_permission_grant;
pub mod insert_role_permission_grant;

use oxidauth_kernel::role_permission_grants::RolePermissionGrant;

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
