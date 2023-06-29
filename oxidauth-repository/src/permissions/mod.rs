pub mod delete_permission;
pub mod insert_permission;
pub mod query_all_permissions;
pub mod query_permission_by_id;
pub mod query_permission_by_parts;
pub mod query_permissions_by_realm;
pub mod update_permission;

use crate::prelude::*;

#[derive(Debug)]
pub struct PermissionRow {
    pub id: Uuid,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
