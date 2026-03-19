use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use crate::error::BoxedError;

pub use super::Role;

#[async_trait]
pub trait ListAllRolesTrait: Send + Sync + 'static {
    async fn list_all_roles(&self, params: &ListAllRoles) -> Result<Vec<Role>, BoxedError>;
}

pub type ListAllRolesService = Arc<dyn ListAllRolesTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllRoles;
