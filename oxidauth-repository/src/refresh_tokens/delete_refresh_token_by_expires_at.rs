use crate::prelude::*;

use super::RefreshTokenRow;

#[async_trait]
pub trait DeleteRefreshTokenByExpiresAt: Send + Sync + 'static {
    async fn delete_refresh_token_by_expires_at(
        &self,
        expires_at: DateTime<Utc>,
    ) -> Result<Vec<RefreshTokenRow>, DeleteRefreshTokenByExpiresAtError>;
}

#[derive(Debug)]
pub struct DeleteRefreshTokenByExpiresAtError {}
