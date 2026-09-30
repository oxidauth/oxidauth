use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use super::Role;
pub use crate::error::BoxedError;

#[async_trait]
pub trait FindRoleByNameServiceTrait: Send + Sync + 'static {
    async fn find_role_by_name(&self, params: &FindRoleByName) -> Result<Role, BoxedError>;
}

pub type FindRoleByNameService = Arc<dyn FindRoleByNameServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRoleByName {
    pub role: String,
}
