use async_trait::async_trait;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::error::BoxedError;

pub use super::Role;

#[async_trait]
pub trait FindRoleByIdTrait: Send + Sync + 'static {
    async fn find_role_by_id(&self, params: &FindRoleById) -> Result<Role, BoxedError>;
}

pub type FindRoleByIdService = Arc<dyn FindRoleByIdTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRoleById {
    pub role_id: Uuid,
}
