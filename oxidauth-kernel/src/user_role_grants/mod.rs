use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::roles::Role;

pub mod create_user_role_grant;
pub mod delete_user_role_grant;
pub mod list_user_role_grants_by_user_id;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRole {
    pub role: Role,
    pub grant: UserRoleGrant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
