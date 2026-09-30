use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use super::Permission;
use crate::error::BoxedError;

#[async_trait]
pub trait FindPermissionByPartsServiceTrait: Send + Sync + 'static {
    async fn find_permission_by_parts(
        &self,
        params: &FindPermissionByParts,
    ) -> Result<Permission, BoxedError>;
}

pub type FindPermissionByPartsService = Arc<dyn FindPermissionByPartsServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPermissionByParts {
    pub permission: String,
}
