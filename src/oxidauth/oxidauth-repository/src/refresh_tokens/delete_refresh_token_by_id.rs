pub use oxidauth_kernel::refresh_tokens::{
    RefreshToken,
    delete_refresh_token_by_id::DeleteRefreshTokenById,
};

pub use crate::prelude::*;

#[async_trait]
pub trait DeleteRefreshTokenByIdQuery: Send + Sync + 'static {
    async fn delete_refresh_token_by_id(
        &self,
        params: &DeleteRefreshTokenById,
    ) -> Result<RefreshToken, BoxedError>;
}
