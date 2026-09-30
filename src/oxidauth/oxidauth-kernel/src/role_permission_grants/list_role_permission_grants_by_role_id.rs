use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::{RolePermission, RolePermissionGrant};
use crate::error::BoxedError;

#[async_trait]
pub trait ListRolePermissionGrantsByRoleIdServiceTrait: Send + Sync + 'static {
    async fn list_role_permission_grants_by_role_id(
        &self,
        params: &ListRolePermissionGrantsByRoleId,
    ) -> Result<Vec<RolePermission>, BoxedError>;
}

pub type ListRolePermissionGrantsByRoleIdService =
    Arc<dyn ListRolePermissionGrantsByRoleIdServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRolePermissionGrantsByRoleId {
    pub role_id: Uuid,
}
