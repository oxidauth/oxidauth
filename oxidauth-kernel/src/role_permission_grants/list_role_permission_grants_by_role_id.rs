use async_trait::async_trait;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::{RolePermission, RolePermissionGrant};

#[async_trait]
pub trait ListRolePermissionGrantsByRoleIdTrait: Send + Sync + 'static {
    async fn list_role_permission_grants_by_role_id(
        &self,
        params: &ListRolePermissionGrantsByRoleId,
    ) -> Result<Vec<RolePermission>, BoxedError>;
}

pub type ListRolePermissionGrantsByRoleIdService = Arc<dyn ListRolePermissionGrantsByRoleIdTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRolePermissionGrantsByRoleId {
    pub role_id: Uuid,
}
