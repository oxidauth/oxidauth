pub mod delete_permission_by_id;
pub mod insert_permission;
pub mod query_all_permissions;
pub mod query_permission_by_id;
pub mod query_permission_by_parts;
pub mod query_permissions_by_realm;
pub mod update_permission;

use oxidauth_repository::permissions::PermissionRow as RepoPermissionRow;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PermissionRow {
    pub id: Uuid,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PermissionRow> for RepoPermissionRow {
    fn from(value: PermissionRow) -> Self {
        Self {
            id: value.id,
            realm: value.realm,
            resource: value.resource,
            action: value.action,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
