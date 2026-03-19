use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::{
    Authority, AuthoritySettings, AuthorityStatus, AuthorityStrategy,
};

#[async_trait]
pub trait UpdateAuthorityTrait: Send + Sync + 'static {
    async fn update_authority(
        &self,
        params: &mut UpdateAuthority,
    ) -> Result<Authority, BoxedError>;
}

pub type UpdateAuthorityService = Arc<dyn UpdateAuthorityTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAuthority {
    pub id: Option<Uuid>,
    pub name: String,
    pub client_key: Option<Uuid>,
    pub status: Option<AuthorityStatus>,
    pub strategy: AuthorityStrategy,
    pub settings: AuthoritySettings,
    pub params: Value,
}
