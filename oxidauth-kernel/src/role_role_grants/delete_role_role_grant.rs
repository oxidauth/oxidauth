use async_trait::async_trait;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::RoleRoleGrant;

#[async_trait]
pub trait DeleteRoleRoleGrantTrait: Send + Sync + 'static {
    async fn delete_role_role_grant(
        &self,
        params: &DeleteRoleRoleGrant,
    ) -> Result<RoleRoleGrant, BoxedError>;
}

pub type DeleteRoleRoleGrantService = Arc<dyn DeleteRoleRoleGrantTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}
