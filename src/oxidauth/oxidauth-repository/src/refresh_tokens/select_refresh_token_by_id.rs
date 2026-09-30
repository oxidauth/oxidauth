pub use oxidauth_kernel::refresh_tokens::{
    RefreshToken,
    find_refresh_token_by_id::FindRefreshTokenById,
};

pub use crate::prelude::*;

#[async_trait]
pub trait SelectRefreshTokenByIdQuery: Send + Sync + 'static {
    async fn select_refresh_token_by_id(
        &self,
        params: &FindRefreshTokenById,
    ) -> Result<RefreshToken, BoxedError>;
}
