use async_trait::async_trait;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use crate::error::BoxedError;

pub use super::Role;

#[async_trait]
pub trait FindRoleByNameTrait: Send + Sync + 'static {
    async fn find_role_by_name(&self, params: &FindRoleByName) -> Result<Role, BoxedError>;
}

pub type FindRoleByNameService = Arc<dyn FindRoleByNameTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRoleByName {
    pub role: String,
}
