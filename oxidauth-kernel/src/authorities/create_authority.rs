use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::BoxedError, JsonValue};

pub use super::{
    Authority, AuthoritySettings, AuthorityStatus, AuthorityStrategy,
};

#[async_trait]
pub trait CreateAuthorityTrait: Send + Sync + 'static {
    async fn create_authority(
        &self,
        params: &mut CreateAuthority,
    ) -> Result<Authority, BoxedError>;
}

pub type CreateAuthorityService = Arc<dyn CreateAuthorityTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthority {
    pub name: String,
    pub client_key: Option<Uuid>,
    pub status: Option<AuthorityStatus>,
    pub strategy: AuthorityStrategy,
    pub settings: AuthoritySettings,
    pub params: JsonValue,
}
