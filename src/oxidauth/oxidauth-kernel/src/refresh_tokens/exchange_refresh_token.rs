use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::RefreshToken;
use crate::{auth::authenticate::AuthenticateResponse, error::BoxedError};

#[async_trait]
pub trait ExchangeRefreshTokenServiceTrait: Send + Sync + 'static {
    async fn exchange_refresh_token(
        &self,
        params: &ExchangeRefreshToken,
    ) -> Result<AuthenticateResponse, BoxedError>;
}

pub type ExchangeRefreshTokenService = Arc<dyn ExchangeRefreshTokenServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeRefreshToken {
    pub refresh_token: Uuid,
}
