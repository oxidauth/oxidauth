use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::BoxedError;

pub use super::Permission;

#[async_trait]
pub trait DeletePermissionTrait: Send + Sync + 'static {
    async fn delete_permission(
        &self,
        params: &DeletePermission,
    ) -> Result<Permission, BoxedError>;
}

pub type DeletePermissionService = Arc<dyn DeletePermissionTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePermission {
    pub permission: String,
}
