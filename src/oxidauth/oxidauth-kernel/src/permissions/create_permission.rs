use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use super::Permission;
use crate::error::BoxedError;

#[async_trait]
pub trait CreatePermissionServiceTrait: Send + Sync + 'static {
    async fn create_permission(&self, params: &CreatePermission) -> Result<Permission, BoxedError>;
}

pub type CreatePermissionService = Arc<dyn CreatePermissionServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePermission {
    pub permission: String,
}
