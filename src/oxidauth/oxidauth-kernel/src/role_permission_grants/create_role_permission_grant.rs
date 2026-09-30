use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::{RolePermission, RolePermissionGrant};
use crate::error::BoxedError;

#[async_trait]
pub trait CreateRolePermissionGrantServiceTrait: Send + Sync + 'static {
    async fn create_role_permission_grant(
        &self,
        params: &CreateRolePermissionGrant,
    ) -> Result<RolePermission, BoxedError>;
}

pub type CreateRolePermissionGrantService = Arc<dyn CreateRolePermissionGrantServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRolePermissionGrant {
    pub role_id: Uuid,
    pub permission: String,
}
