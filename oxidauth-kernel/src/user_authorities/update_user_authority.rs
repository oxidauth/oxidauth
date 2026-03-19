use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{error::BoxedError, JsonValue};

pub use super::UserAuthority;

#[async_trait]
pub trait UpdateUserAuthorityTrait: Send + Sync + 'static {
    async fn update_user_authority(
        &self,
        params: &UpdateUserAuthority,
    ) -> Result<UserAuthority, BoxedError>;
}

pub type UpdateUserAuthorityService = Arc<dyn UpdateUserAuthorityTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub params: JsonValue,
}
