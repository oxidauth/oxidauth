use crate::dev_prelude::*;

use super::*;

#[derive(Debug)]
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

#[async_trait]
pub trait UserCreateService: Send + Sync + 'static {
    async fn create_user(&self, params: &UserCreate) -> Result<User, CreateUserError>;
}

#[derive(Debug)]
pub struct CreateUserError {}
