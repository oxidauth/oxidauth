pub mod find_user_by_id;
pub mod user_create;

use crate::dev_prelude::*;

pub struct User {
    pub id: Uuid,
    pub kind: UserKind,
    pub status: UserStatus,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum UserKind {
    Human,
    Api,
}

#[derive(Debug)]
pub enum UserStatus {
    Enabled,
    Disabled,
    Locked,
}
