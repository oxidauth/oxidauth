pub mod insert_user;
pub mod query_user_by_email;
pub mod query_user_by_id;
pub mod query_user_by_username;

pub use crate::prelude::*;

#[derive(Debug)]
pub struct UserRow {
    pub id: Uuid,
    pub kind: String,
    pub status: String,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
