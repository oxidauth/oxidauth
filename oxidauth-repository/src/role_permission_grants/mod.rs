pub mod delete_role_permission_grant;
pub mod insert_role_permission_grant;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct RolePermissionGrantRow {
    pub id: Uuid, // @GEORGE -- I added this here assuming it will have an id
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
