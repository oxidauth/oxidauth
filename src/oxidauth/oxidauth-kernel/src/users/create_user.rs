use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub use super::{User, UserKind, UserStatus};
use crate::error::BoxedError;

#[async_trait]
pub trait CreateUserServiceTrait: Send + Sync + 'static {
    async fn create_user(&self, params: &CreateUser) -> Result<User, BoxedError>;
}

pub type CreateUserService = Arc<dyn CreateUserServiceTrait>;

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
