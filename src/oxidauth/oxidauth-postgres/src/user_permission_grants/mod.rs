pub mod delete_user_permission_grant;
pub mod insert_user_permission_grant;
pub mod select_user_permission_grants_by_user_id;

use oxidauth_kernel::{
    permissions::Permission,
    user_permission_grants::{UserPermission, UserPermissionGrant},
};

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgUserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgUserPermissionGrant> for UserPermissionGrant {
    fn from(value: PgUserPermissionGrant) -> Self {
        Self {
            user_id: value.user_id,
            permission_id: value.permission_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PgUserPermission {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub permission_created_at: DateTime<Utc>,
    pub permission_updated_at: DateTime<Utc>,
}

impl From<PgUserPermission> for UserPermission {
    fn from(value: PgUserPermission) -> Self {
        Self {
            permission: Permission {
                id: value.permission_id,
                realm: value.realm,
                resource: value.resource,
                action: value.action,
                created_at: value.permission_created_at,
                updated_at: value.permission_updated_at,
            },
            grant: UserPermissionGrant {
                user_id: value.user_id,
                permission_id: value.permission_id,
                created_at: value.created_at,
                updated_at: value.updated_at,
            },
        }
    }
}
