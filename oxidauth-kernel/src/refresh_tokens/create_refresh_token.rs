use crate::dev_prelude::*;

pub use super::RefreshToken;

#[async_trait]
pub trait CreateRefreshTokenTrait: Send + Sync + 'static {
    async fn create_refresh_token(
        &self,
        params: &CreateRefreshToken,
    ) -> Result<RefreshToken, BoxedError>;
}

pub type CreateRefreshTokenService = Arc<dyn CreateRefreshTokenTrait>;

#[derive(Debug, Deserialize)]
pub struct CreateRefreshToken {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub expires_at: DateTime<Utc>,
}
