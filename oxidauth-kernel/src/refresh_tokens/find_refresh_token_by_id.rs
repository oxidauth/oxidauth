use crate::dev_prelude::*;

pub use super::RefreshToken;

#[async_trait]
pub trait FindRefreshTokenByIdTrait: Send + Sync + 'static {
    async fn find_refresh_token_by_id(
        &self,
        params: &FindRefreshTokenById,
    ) -> Result<RefreshToken, BoxedError>;
}

pub type FindRefreshTokenByIdService = Arc<dyn FindRefreshTokenByIdTrait>;

#[derive(Debug, Deserialize)]
pub struct FindRefreshTokenById {
    pub refresh_token_id: Uuid,
}
