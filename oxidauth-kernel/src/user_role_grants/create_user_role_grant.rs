use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::UserRole;

#[async_trait]
pub trait CreateUserRoleGrantTrait: Send + Sync + 'static {
    async fn create_user_role_grant(
        &self,
        params: &CreateUserRoleGrant,
    ) -> Result<UserRole, BoxedError>;
}

pub type CreateUserRoleGrantService = Arc<dyn CreateUserRoleGrantTrait>;

#[derive(Debug, Deserialize)]
pub struct CreateUserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
}
