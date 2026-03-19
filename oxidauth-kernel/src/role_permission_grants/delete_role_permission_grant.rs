use async_trait::async_trait;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::{RolePermission, RolePermissionGrant};

#[async_trait]
pub trait DeleteRolePermissionGrantTrait: Send + Sync + 'static {
    async fn delete_role_permission_grant(
        &self,
        params: &DeleteRolePermissionGrant,
    ) -> Result<RolePermission, BoxedError>;
}

pub type DeleteRolePermissionGrantService = Arc<dyn DeleteRolePermissionGrantTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRolePermissionGrant {
    pub role_id: Uuid,
    pub permission: String,
}
