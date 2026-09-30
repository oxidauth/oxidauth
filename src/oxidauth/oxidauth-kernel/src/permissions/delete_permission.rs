use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use super::Permission;
use crate::error::BoxedError;

#[async_trait]
pub trait DeletePermissionServiceTrait: Send + Sync + 'static {
    async fn delete_permission(&self, params: &DeletePermission) -> Result<Permission, BoxedError>;
}

pub type DeletePermissionService = Arc<dyn DeletePermissionServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePermission {
    pub permission: String,
}
