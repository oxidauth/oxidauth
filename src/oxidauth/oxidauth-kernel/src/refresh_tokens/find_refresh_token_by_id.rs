use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

pub use super::RefreshToken;
use crate::error::BoxedError;

#[async_trait]
pub trait FindRefreshTokenByIdServiceTrait: Send + Sync + 'static {
    async fn find_refresh_token_by_id(
        &self,
        params: &FindRefreshTokenById,
    ) -> Result<RefreshToken, BoxedError>;
}

pub type FindRefreshTokenByIdService = Arc<dyn FindRefreshTokenByIdServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct FindRefreshTokenById {
    pub refresh_token_id: Uuid,
}
