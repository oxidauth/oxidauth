use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::{Authority, AuthoritySettings, AuthorityStatus, AuthorityStrategy};
use crate::{JsonValue, error::BoxedError};

#[async_trait]
pub trait CreateAuthorityServiceTrait: Send + Sync + 'static {
    async fn create_authority(&self, params: &mut CreateAuthority)
    -> Result<Authority, BoxedError>;
}

pub type CreateAuthorityService = Arc<dyn CreateAuthorityServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthority {
    pub name: String,
    pub client_key: Option<Uuid>,
    pub status: Option<AuthorityStatus>,
    pub strategy: AuthorityStrategy,
    pub settings: AuthoritySettings,
    pub params: JsonValue,
}
