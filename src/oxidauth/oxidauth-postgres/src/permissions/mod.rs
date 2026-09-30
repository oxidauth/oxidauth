use chrono::{DateTime, Utc};
use oxidauth_kernel::permissions::Permission;
use uuid::Uuid;

pub mod delete_permission;
pub mod insert_permission;
pub mod select_all_permissions;
pub mod select_permission_by_parts;

use crate::Database;

#[derive(Debug, Clone)]
pub struct PgPermissionRepository {
    db: Database,
}

impl PgPermissionRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PgPermission {
    pub id: Uuid,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgPermission> for Permission {
    fn from(value: PgPermission) -> Self {
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
