use crate::dev_prelude::*;

pub use super::RefreshToken;

#[async_trait]
pub trait DeleteRefreshTokenTrait: Send + Sync + 'static {
    async fn delete_refresh_token(
        &self,
        params: &DeleteRefreshToken,
    ) -> Result<RefreshToken, BoxedError>;
}

pub type DeleteRefreshTokenService = Arc<dyn DeleteRefreshTokenTrait>;

#[derive(Debug, Deserialize)]
pub struct DeleteRefreshToken {
    pub refresh_token_id: Uuid,
}
