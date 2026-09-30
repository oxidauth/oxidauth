use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use super::Permission;
use crate::error::BoxedError;

#[async_trait]
pub trait ListAllPermissionsServiceTrait: Send + Sync + 'static {
    async fn list_all_permissions(
        &self,
        params: &ListAllPermissions,
    ) -> Result<Vec<Permission>, BoxedError>;
}

pub type ListAllPermissionsService = Arc<dyn ListAllPermissionsServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllPermissions;
