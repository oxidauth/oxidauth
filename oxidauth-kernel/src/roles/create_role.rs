use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use crate::error::BoxedError;

pub use super::Role;

#[async_trait]
pub trait CreateRoleTrait: Send + Sync + 'static {
    async fn create_role(&self, params: &CreateRole) -> Result<Role, BoxedError>;
}

pub type CreateRoleService = Arc<dyn CreateRoleTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRole {
    pub name: String,
}
