use chrono::{DateTime, Utc};
use oxidauth_kernel::permissions::Permission;
use uuid::Uuid;

pub mod delete_permission;
pub mod insert_permission;
pub mod select_all_permissions;
pub mod select_permission_by_parts;
pub mod update_permission;

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
