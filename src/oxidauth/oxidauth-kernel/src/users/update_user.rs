use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub use super::{User, UserStatus};
use crate::error::BoxedError;

#[async_trait]
pub trait UpdateUserServiceTrait: Send + Sync + 'static {
    async fn update_user(&self, params: &mut UpdateUser) -> Result<User, BoxedError>;
}

pub type UpdateUserService = Arc<dyn UpdateUserServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<UserStatus>,
    pub profile: Option<Value>,
}
