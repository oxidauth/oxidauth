use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::Authority;

#[async_trait]
pub trait FindAuthorityByIdTrait: Send + Sync + 'static {
    async fn find_authority_by_id(
        &self,
        params: &FindAuthorityById,
    ) -> Result<Authority, BoxedError>;
}

pub type FindAuthorityByIdService = Arc<dyn FindAuthorityByIdTrait>;

#[derive(Debug, Deserialize)]
pub struct FindAuthorityById {
    pub authority_id: Uuid,
}
