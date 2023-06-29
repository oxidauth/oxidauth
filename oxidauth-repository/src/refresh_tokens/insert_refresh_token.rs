use crate::prelude::*;

use super::RefreshTokenRow;

#[async_trait]
pub trait InsertRefreshToken: Send + Sync + 'static {
    async fn isnert_refresh_token(
        &self,
        params: &InsertRefreshTokenParams,
    ) -> Result<RefreshTokenRow, InsertRefreshTokenError>;
}

#[derive(Debug)]
pub struct InsertRefreshTokenParams {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct InsertRefreshTokenError {}
