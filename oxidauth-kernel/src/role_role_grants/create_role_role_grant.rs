use async_trait::async_trait;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::RoleRoleGrant;
pub use super::RoleRoleGrantDetail;

#[async_trait]
pub trait CreateRoleRoleGrantTrait: Send + Sync + 'static {
    async fn create_role_role_grant(
        &self,
        params: &CreateRoleRoleGrant,
    ) -> Result<RoleRoleGrantDetail, BoxedError>;
}

pub type CreateRoleRoleGrantService = Arc<dyn CreateRoleRoleGrantTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}
