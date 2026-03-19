use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::{User, UserKind, UserStatus};

#[async_trait]
pub trait CreateUserTrait: Send + Sync + 'static {
    async fn create_user(
        &self,
        params: &CreateUser,
    ) -> Result<User, BoxedError>;
}

pub type CreateUserService = Arc<dyn CreateUserTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub id: Option<Uuid>,
    pub kind: Option<UserKind>,
    pub status: Option<UserStatus>,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Option<Value>,
}
