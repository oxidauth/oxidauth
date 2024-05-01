use oxidauth_kernel::auth_keys::create_auth_key::{
    CreateAuthKeyResponse, InsertAuthKeyParams,
};
pub use oxidauth_kernel::service::Service;

pub use crate::prelude::*;

pub trait InsertAuthKeyQuery:
    for<'a> Service<
    &'a InsertAuthKeyParams,
    Response = CreateAuthKeyResponse,
    Error = BoxedError,
>
{
}

impl<T> InsertAuthKeyQuery for T where
    T: for<'a> Service<
        &'a InsertAuthKeyParams,
        Response = CreateAuthKeyResponse,
        Error = BoxedError,
    >
{
}
