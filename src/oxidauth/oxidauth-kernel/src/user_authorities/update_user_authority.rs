use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::UserAuthority;
use crate::{JsonValue, error::BoxedError};

#[async_trait]
pub trait UpdateUserAuthorityServiceTrait: Send + Sync + 'static {
    async fn update_user_authority(
        &self,
        params: &UpdateUserAuthority,
    ) -> Result<UserAuthority, BoxedError>;
}

pub type UpdateUserAuthorityService = Arc<dyn UpdateUserAuthorityServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub params: JsonValue,
}
