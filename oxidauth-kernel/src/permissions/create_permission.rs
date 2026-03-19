use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::BoxedError;

pub use super::Permission;

#[async_trait]
pub trait CreatePermissionTrait: Send + Sync + 'static {
    async fn create_permission(
        &self,
        params: &CreatePermission,
    ) -> Result<Permission, BoxedError>;
}

pub type CreatePermissionService = Arc<dyn CreatePermissionTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePermission {
    pub permission: String,
}
