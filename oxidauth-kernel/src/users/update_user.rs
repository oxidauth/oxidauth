use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::{User, UserStatus};

#[async_trait]
pub trait UpdateUserTrait: Send + Sync + 'static {
    async fn update_user(
        &self,
        params: &mut UpdateUser,
    ) -> Result<User, BoxedError>;
}

pub type UpdateUserService = Arc<dyn UpdateUserTrait>;

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
