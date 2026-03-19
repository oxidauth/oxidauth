use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::Authority;

#[async_trait]
pub trait FindAuthorityByClientKeyTrait: Send + Sync + 'static {
    async fn find_authority_by_client_key(
        &self,
        params: &FindAuthorityByClientKey,
    ) -> Result<Authority, BoxedError>;
}

pub type FindAuthorityByClientKeyService = Arc<dyn FindAuthorityByClientKeyTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAuthorityByClientKey {
    pub client_key: Uuid,
}
