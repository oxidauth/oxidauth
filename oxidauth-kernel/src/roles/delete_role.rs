use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub use crate::error::BoxedError;

pub use super::Role;

#[async_trait]
pub trait DeleteRoleTrait: Send + Sync + 'static {
    async fn delete_role(&self, params: &DeleteRole) -> Result<Role, BoxedError>;
}

pub type DeleteRoleService = Arc<dyn DeleteRoleTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRole {
    pub role_id: Uuid,
}
