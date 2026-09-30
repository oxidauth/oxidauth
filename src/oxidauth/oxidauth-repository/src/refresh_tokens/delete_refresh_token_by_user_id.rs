pub use oxidauth_kernel::refresh_tokens::{
    RefreshToken,
    delete_refresh_token_by_user_id::DeleteRefreshTokenByUserId,
};

pub use crate::prelude::*;

#[async_trait]
pub trait DeleteRefreshTokenByUserIdQuery: Send + Sync + 'static {
    /// Deletes every refresh token of `user_id`. Returns the deleted rows
    /// (possibly empty); zero matches is a success with an empty `Vec`, never
    /// an error.
    async fn delete_refresh_token_by_user_id(
        &self,
        params: &DeleteRefreshTokenByUserId,
    ) -> Result<Vec<RefreshToken>, BoxedError>;
}
