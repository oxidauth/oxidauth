use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::UserAuthority;

#[async_trait]
pub trait DeleteUserAuthorityTrait: Send + Sync + 'static {
    async fn delete_user_authority(
        &self,
        params: &DeleteUserAuthority,
    ) -> Result<UserAuthority, BoxedError>;
}

pub type DeleteUserAuthorityService = Arc<dyn DeleteUserAuthorityTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}
