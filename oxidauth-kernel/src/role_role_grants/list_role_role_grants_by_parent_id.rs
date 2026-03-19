use async_trait::async_trait;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::RoleRoleGrant;
pub use super::RoleRoleGrantDetail;

#[async_trait]
pub trait ListRoleRoleGrantsByParentIdTrait: Send + Sync + 'static {
    async fn list_role_role_grants_by_parent_id(
        &self,
        params: &ListRoleRoleGrantsByParentId,
    ) -> Result<Vec<RoleRoleGrantDetail>, BoxedError>;
}

pub type ListRoleRoleGrantsByParentIdService = Arc<dyn ListRoleRoleGrantsByParentIdTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRoleRoleGrantsByParentId {
    pub parent_id: Uuid,
}
