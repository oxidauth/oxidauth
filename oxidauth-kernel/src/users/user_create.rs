use std::sync::Arc;

// use crate::dev_prelude::*;

pub use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreate {
    pub id: Option<Uuid>,
    pub kind: Option<UserKind>,
    pub status: Option<UserStatus>,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Option<Value>,
}

pub const EMPTY_STR: &str = "";

impl Default for UserCreate {
    fn default() -> Self {
        Self {
            id: None,
            kind: Some(UserKind::Human),
            status: Some(UserStatus::Enabled),
            username: EMPTY_STR.to_string(),
            email: None,
            first_name: None,
            last_name: None,
            profile: None,
        }
    }
}

impl From<&str> for UserCreate {
    fn from(username: &str) -> Self {
        Self {
            username: username.to_owned(),
            ..Default::default()
        }
    }
}

pub type CreateUserService = Arc<dyn CreateUserTrait>;

#[async_trait]
pub trait CreateUserTrait: Send + Sync + 'static {
    async fn create_user(
        &self,
        params: &UserCreate,
    ) -> Result<User, CreateUserError>;
}

#[derive(Debug, Serialize)]
pub struct CreateUserError {}
