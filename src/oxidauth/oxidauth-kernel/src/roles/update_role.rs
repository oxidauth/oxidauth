use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::Role;
pub use crate::error::BoxedError;

#[async_trait]
pub trait UpdateRoleServiceTrait: Send + Sync + 'static {
    async fn update_role(&self, params: &UpdateRole) -> Result<Role, BoxedError>;
}

pub type UpdateRoleService = Arc<dyn UpdateRoleServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRole {
    pub role_id: Option<Uuid>,
    pub name: String,
}
