use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use super::Role;
pub use crate::error::BoxedError;

#[async_trait]
pub trait CreateRoleServiceTrait: Send + Sync + 'static {
    async fn create_role(&self, params: &CreateRole) -> Result<Role, BoxedError>;
}

pub type CreateRoleService = Arc<dyn CreateRoleServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRole {
    pub name: String,
}
