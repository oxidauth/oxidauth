use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::BoxedError;

pub use super::Permission;

#[async_trait]
pub trait FindPermissionByPartsTrait: Send + Sync + 'static {
    async fn find_permission_by_parts(
        &self,
        params: &FindPermissionByParts,
    ) -> Result<Permission, BoxedError>;
}

pub type FindPermissionByPartsService = Arc<dyn FindPermissionByPartsTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPermissionByParts {
    pub permission: String,
}
