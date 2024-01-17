use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::roles::Role;

pub mod create_user_role_grant;
pub mod list_user_role_grants_by_user_id;

#[derive(Debug, Serialize)]
pub struct UserRole {
    pub role: Role,
    pub grant: UserRoleGrant,
}

#[derive(Debug, Serialize)]
pub struct UserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
