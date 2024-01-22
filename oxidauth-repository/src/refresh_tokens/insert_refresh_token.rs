pub use oxidauth_kernel::refresh_tokens::create_refresh_token::CreateRefreshToken;
pub use oxidauth_kernel::{service::Service, refresh_tokens::RefreshToken};

pub use crate::prelude::*;

pub trait InsertRefreshTokenQuery:
    for<'a> Service<&'a CreateRefreshToken, Response = RefreshToken, Error = BoxedError>
{
}

impl<T> InsertRefreshTokenQuery for T where
    T: for<'a> Service<&'a CreateRefreshToken, Response = RefreshToken, Error = BoxedError>
{
}
