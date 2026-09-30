use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

pub use super::RefreshToken;
use crate::error::BoxedError;

#[async_trait]
pub trait DeleteRefreshTokenByIdServiceTrait: Send + Sync + 'static {
    async fn delete_refresh_token_by_id(
        &self,
        params: &DeleteRefreshTokenById,
    ) -> Result<RefreshToken, BoxedError>;
}

pub type DeleteRefreshTokenByIdService = Arc<dyn DeleteRefreshTokenByIdServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct DeleteRefreshTokenById {
    pub refresh_token_id: Uuid,
}
