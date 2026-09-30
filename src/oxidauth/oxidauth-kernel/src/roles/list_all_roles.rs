use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use super::Role;
pub use crate::error::BoxedError;

#[async_trait]
pub trait ListAllRolesServiceTrait: Send + Sync + 'static {
    async fn list_all_roles(&self, params: &ListAllRoles) -> Result<Vec<Role>, BoxedError>;
}

pub type ListAllRolesService = Arc<dyn ListAllRolesServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllRoles;
