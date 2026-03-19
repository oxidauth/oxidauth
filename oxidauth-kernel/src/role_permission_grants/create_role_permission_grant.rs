use async_trait::async_trait;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::{RolePermission, RolePermissionGrant};

#[async_trait]
pub trait CreateRolePermissionGrantTrait: Send + Sync + 'static {
    async fn create_role_permission_grant(
        &self,
        params: &CreateRolePermissionGrant,
    ) -> Result<RolePermission, BoxedError>;
}

pub type CreateRolePermissionGrantService = Arc<dyn CreateRolePermissionGrantTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRolePermissionGrant {
    pub role_id: Uuid,
    pub permission: String,
}
