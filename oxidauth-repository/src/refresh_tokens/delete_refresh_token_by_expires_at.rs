use crate::prelude::*;

use super::RefreshTokenRow;

#[async_trait]
pub trait DeleteRefreshToken: Send + Sync + 'static {
    async fn delete_refresh_token_by_expires_at(
        &self,
        refresh_token_id: Uuid,
    ) -> Result<RefreshTokenRow, DeleteRefreshTokenError>;
}

#[derive(Debug)]
pub struct DeleteRefreshTokenError {}
