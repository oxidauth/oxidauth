use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

pub use super::UserRole;
use crate::error::BoxedError;

#[async_trait]
pub trait CreateUserRoleGrantServiceTrait: Send + Sync + 'static {
    async fn create_user_role_grant(
        &self,
        params: &CreateUserRoleGrant,
    ) -> Result<UserRole, BoxedError>;
}

pub type CreateUserRoleGrantService = Arc<dyn CreateUserRoleGrantServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct CreateUserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
}
