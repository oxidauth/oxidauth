use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::Authority;

#[async_trait]
pub trait DeleteAuthorityTrait: Send + Sync + 'static {
    async fn delete_authority(
        &self,
        params: &DeleteAuthority,
    ) -> Result<Authority, BoxedError>;
}

pub type DeleteAuthorityService = Arc<dyn DeleteAuthorityTrait>;

#[derive(Debug, Deserialize)]
pub struct DeleteAuthority {
    pub authority_id: Uuid,
}
