pub use crate::prelude::*;

use super::UserRow;

#[async_trait]
pub trait InsertUser: Send + Sync + 'static {
    async fn insert_user(&self, user_insert: &InsertUserParams)
        -> Result<UserRow, InsertUserError>;
}

#[derive(Debug)]
pub struct InsertUserParams {
    pub id: Option<Uuid>,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Option<Value>,
    pub kind: Option<String>,
    pub status: Option<String>,
}

pub const HUMAN: &str = "human";
pub const ENABLED: &str = "enabled";

impl Default for InsertUserParams {
    fn default() -> Self {
        Self {
            id: None,
            kind: Some(HUMAN.to_string()),
            status: Some(ENABLED.to_string()),
            username: EMPTY_STR.to_string(),
            email: None,
            first_name: None,
            last_name: None,
            profile: None,
        }
    }
}

impl From<String> for InsertUserParams {
    fn from(username: String) -> Self {
        Self {
            username,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct InsertUserError {}
