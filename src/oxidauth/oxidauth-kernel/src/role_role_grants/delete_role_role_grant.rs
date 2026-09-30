use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::RoleRoleGrant;
use crate::error::BoxedError;

#[async_trait]
pub trait DeleteRoleRoleGrantServiceTrait: Send + Sync + 'static {
    async fn delete_role_role_grant(
        &self,
        params: &DeleteRoleRoleGrant,
    ) -> Result<RoleRoleGrant, BoxedError>;
}

pub type DeleteRoleRoleGrantService = Arc<dyn DeleteRoleRoleGrantServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}
