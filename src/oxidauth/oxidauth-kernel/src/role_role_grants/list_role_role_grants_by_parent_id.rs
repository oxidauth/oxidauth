use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::{RoleRoleGrant, RoleRoleGrantDetail};
use crate::error::BoxedError;

#[async_trait]
pub trait ListRoleRoleGrantsByParentIdServiceTrait: Send + Sync + 'static {
    async fn list_role_role_grants_by_parent_id(
        &self,
        params: &ListRoleRoleGrantsByParentId,
    ) -> Result<Vec<RoleRoleGrantDetail>, BoxedError>;
}

pub type ListRoleRoleGrantsByParentIdService = Arc<dyn ListRoleRoleGrantsByParentIdServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRoleRoleGrantsByParentId {
    pub parent_id: Uuid,
}
