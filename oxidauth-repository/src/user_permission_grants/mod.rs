pub mod delete_user_permission_grant;
pub mod insert_user_permission_grant;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct UserPermissionGrantRow {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
