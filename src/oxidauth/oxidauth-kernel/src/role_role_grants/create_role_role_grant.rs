use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::{RoleRoleGrant, RoleRoleGrantDetail};
use crate::error::BoxedError;

#[async_trait]
pub trait CreateRoleRoleGrantServiceTrait: Send + Sync + 'static {
    async fn create_role_role_grant(
        &self,
        params: &CreateRoleRoleGrant,
    ) -> Result<RoleRoleGrantDetail, BoxedError>;
}

pub type CreateRoleRoleGrantService = Arc<dyn CreateRoleRoleGrantServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}
