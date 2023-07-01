use crate::prelude::*;

use super::RefreshTokenRow;

#[async_trait]
pub trait DeleteRefreshTokenByExpiresAt: Send + Sync + 'static {
    async fn delete_refresh_token_by_expires_at(
        &self,
        refresh_token_id: Uuid,
    ) -> Result<RefreshTokenRow, DeleteRefreshTokenByExpiresAtError>;
}

#[derive(Debug)]
pub struct DeleteRefreshTokenByExpiresAtError {}

// @GEORGE - not sure about this one - the method was delete by expires at but the argument is id, not sure if this is supposed to be delete by id
