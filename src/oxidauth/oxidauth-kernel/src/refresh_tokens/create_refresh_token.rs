use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

pub use super::RefreshToken;
use crate::error::BoxedError;

#[async_trait]
pub trait CreateRefreshTokenServiceTrait: Send + Sync + 'static {
    async fn create_refresh_token(
        &self,
        params: &CreateRefreshToken,
    ) -> Result<RefreshToken, BoxedError>;
}

pub type CreateRefreshTokenService = Arc<dyn CreateRefreshTokenServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct CreateRefreshToken {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    /// See [`RefreshToken::totp_validated`]. Explicit, not defaulted: every mint
    /// site must decide whether the session it is opening still owes a code.
    pub totp_validated: bool,
    pub expires_at: DateTime<Utc>,
}
