use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::BoxedError;

pub use super::Permission;

#[async_trait]
pub trait ListAllPermissionsTrait: Send + Sync + 'static {
    async fn list_all_permissions(
        &self,
        params: &ListAllPermissions,
    ) -> Result<Vec<Permission>, BoxedError>;
}

pub type ListAllPermissionsService = Arc<dyn ListAllPermissionsTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllPermissions;
