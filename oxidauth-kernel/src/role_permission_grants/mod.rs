use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::permissions::Permission;

pub mod create_role_permission_grant;
pub mod delete_role_permission_grant;
pub mod list_role_permission_grants_by_role_id;

#[derive(Debug, Serialize, Deserialize)]
pub struct RolePermission {
    pub permission: Permission,
    pub grant: RolePermissionGrant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RolePermissionGrant {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
