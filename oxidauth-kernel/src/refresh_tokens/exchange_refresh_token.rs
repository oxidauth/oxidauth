use crate::dev_prelude::*;

use crate::auth::authenticate::AuthenticateResponse;

pub use super::RefreshToken;

#[async_trait]
pub trait ExchangeRefreshTokenTrait: Send + Sync + 'static {
    async fn exchange_refresh_token(
        &self,
        params: &ExchangeRefreshToken,
    ) -> Result<AuthenticateResponse, BoxedError>;
}

pub type ExchangeRefreshTokenService = Arc<dyn ExchangeRefreshTokenTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeRefreshToken {
    pub refresh_token: Uuid,
}
