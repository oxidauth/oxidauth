use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

pub use super::Authority;
use crate::error::BoxedError;

#[async_trait]
pub trait FindAuthorityByIdServiceTrait: Send + Sync + 'static {
    async fn find_authority_by_id(
        &self,
        params: &FindAuthorityById,
    ) -> Result<Authority, BoxedError>;
}

pub type FindAuthorityByIdService = Arc<dyn FindAuthorityByIdServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct FindAuthorityById {
    pub authority_id: Uuid,
}
