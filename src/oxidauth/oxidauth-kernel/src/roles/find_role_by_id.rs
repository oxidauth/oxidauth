use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::Role;
pub use crate::error::BoxedError;

#[async_trait]
pub trait FindRoleByIdServiceTrait: Send + Sync + 'static {
    async fn find_role_by_id(&self, params: &FindRoleById) -> Result<Role, BoxedError>;
}

pub type FindRoleByIdService = Arc<dyn FindRoleByIdServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRoleById {
    pub role_id: Uuid,
}
