use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub use crate::error::BoxedError;

pub use super::Role;

#[async_trait]
pub trait UpdateRoleTrait: Send + Sync + 'static {
    async fn update_role(&self, params: &UpdateRole) -> Result<Role, BoxedError>;
}

pub type UpdateRoleService = Arc<dyn UpdateRoleTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRole {
    pub role_id: Option<Uuid>,
    pub name: String,
}
