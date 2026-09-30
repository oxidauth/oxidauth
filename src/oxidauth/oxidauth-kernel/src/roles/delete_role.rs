use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::Role;
pub use crate::error::BoxedError;

#[async_trait]
pub trait DeleteRoleServiceTrait: Send + Sync + 'static {
    async fn delete_role(&self, params: &DeleteRole) -> Result<Role, BoxedError>;
}

pub type DeleteRoleService = Arc<dyn DeleteRoleServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRole {
    pub role_id: Uuid,
}
