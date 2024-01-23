pub use oxidauth_kernel::refresh_tokens::find_refresh_token_by_id::FindRefreshTokenById;
pub use oxidauth_kernel::{service::Service, refresh_tokens::RefreshToken};

pub use crate::prelude::*;

pub trait SelectRefreshTokenByIdQuery:
    for<'a> Service<&'a FindRefreshTokenById, Response = RefreshToken, Error = BoxedError>
{
}

impl<T> SelectRefreshTokenByIdQuery for T where
    T: for<'a> Service<&'a FindRefreshTokenById, Response = RefreshToken, Error = BoxedError>
{
}

