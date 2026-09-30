use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::UserPermission;
use crate::error::BoxedError;

#[async_trait]
pub trait CreateUserPermissionGrantServiceTrait: Send + Sync + 'static {
    async fn create_user_permission_grant(
        &self,
        params: &CreateUserPermission,
    ) -> Result<UserPermission, BoxedError>;
}

pub type CreateUserPermissionGrantService = Arc<dyn CreateUserPermissionGrantServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserPermission {
    pub user_id: Uuid,
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}
