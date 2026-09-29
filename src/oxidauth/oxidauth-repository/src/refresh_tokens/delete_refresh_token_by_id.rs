pub use oxidauth_kernel::refresh_tokens::delete_refresh_token_by_id::DeleteRefreshTokenById;
pub use oxidauth_kernel::{refresh_tokens::RefreshToken, service::Service};

pub use crate::prelude::*;

pub trait DeleteRefreshTokenByIdQuery:
    for<'a> Service<&'a DeleteRefreshTokenById, Response = RefreshToken, Error = BoxedError>
{
}

impl<T> DeleteRefreshTokenByIdQuery for T where
    T: for<'a> Service<&'a DeleteRefreshTokenById, Response = RefreshToken, Error = BoxedError>
{
}
