pub use oxidauth_kernel::refresh_tokens::{RefreshToken, create_refresh_token::CreateRefreshToken};

pub use crate::prelude::*;

#[async_trait]
pub trait InsertRefreshTokenQuery: Send + Sync + 'static {
    async fn insert_refresh_token(
        &self,
        params: &CreateRefreshToken,
    ) -> Result<RefreshToken, BoxedError>;
}
