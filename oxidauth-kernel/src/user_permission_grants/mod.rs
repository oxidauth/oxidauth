use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::permissions::Permission;

pub mod create_user_permission_grant;
pub mod delete_user_permission_grant;
pub mod list_user_permission_grants_by_user_id;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPermission {
    pub permission: Permission,
    pub grant: UserPermissionGrant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
