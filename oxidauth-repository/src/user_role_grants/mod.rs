pub mod delete_user_role_grant;
pub mod insert_user_role_grant;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct UserRoleGrantRow {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
