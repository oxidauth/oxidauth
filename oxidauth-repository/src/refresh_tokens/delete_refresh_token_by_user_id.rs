pub use oxidauth_kernel::refresh_tokens::delete_refresh_token_by_user_id::DeleteRefreshTokenByUserId;
pub use oxidauth_kernel::{refresh_tokens::RefreshToken, service::Service};

pub use crate::prelude::*;

pub trait DeleteRefreshTokenByUserIdQuery:
    for<'a> Service<&'a DeleteRefreshTokenByUserId, Response = RefreshToken, Error = BoxedError>
{
}

impl<T> DeleteRefreshTokenByUserIdQuery for T where
    T: for<'a> Service<&'a DeleteRefreshTokenByUserId, Response = RefreshToken, Error = BoxedError>
{
}
