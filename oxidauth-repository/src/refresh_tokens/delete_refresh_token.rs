pub use oxidauth_kernel::refresh_tokens::delete_refresh_token::DeleteRefreshToken;
pub use oxidauth_kernel::{refresh_tokens::RefreshToken, service::Service};

pub use crate::prelude::*;

pub trait DeleteRefreshTokenQuery:
    for<'a> Service<
    &'a DeleteRefreshToken,
    Response = RefreshToken,
    Error = BoxedError,
>
{
}

impl<T> DeleteRefreshTokenQuery for T where
    T: for<'a> Service<
        &'a DeleteRefreshToken,
        Response = RefreshToken,
        Error = BoxedError,
    >
{
}
