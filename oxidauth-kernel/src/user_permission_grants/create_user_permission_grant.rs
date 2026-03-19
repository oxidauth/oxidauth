use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::UserPermission;

#[async_trait]
pub trait CreateUserPermissionGrantTrait: Send + Sync + 'static {
    async fn create_user_permission_grant(
        &self,
        params: &CreateUserPermission,
    ) -> Result<UserPermission, BoxedError>;
}

pub type CreateUserPermissionGrantService = Arc<dyn CreateUserPermissionGrantTrait>;

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
