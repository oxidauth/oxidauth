use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

pub use super::Authority;
use crate::error::BoxedError;

#[async_trait]
pub trait DeleteAuthorityServiceTrait: Send + Sync + 'static {
    async fn delete_authority(&self, params: &DeleteAuthority) -> Result<Authority, BoxedError>;
}

pub type DeleteAuthorityService = Arc<dyn DeleteAuthorityServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct DeleteAuthority {
    pub authority_id: Uuid,
}
