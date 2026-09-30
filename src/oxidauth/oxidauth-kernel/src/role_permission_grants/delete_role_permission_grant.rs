use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::{RolePermission, RolePermissionGrant};
use crate::error::BoxedError;

#[async_trait]
pub trait DeleteRolePermissionGrantServiceTrait: Send + Sync + 'static {
    async fn delete_role_permission_grant(
        &self,
        params: &DeleteRolePermissionGrant,
    ) -> Result<RolePermission, BoxedError>;
}

pub type DeleteRolePermissionGrantService = Arc<dyn DeleteRolePermissionGrantServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRolePermissionGrant {
    pub role_id: Uuid,
    pub permission: String,
}
