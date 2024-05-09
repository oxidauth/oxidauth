pub use oxidauth_kernel::service::Service;
use oxidauth_kernel::totp_secrets::create_totp_secret::{
    CreateTotpSecretResponse, InsertTotpSecretParams,
};

pub use crate::prelude::*;

pub trait InsertTotpSecretQuery:
    for<'a> Service<
    &'a InsertTotpSecretParams,
    Response = CreateTotpSecretResponse,
    Error = BoxedError,
>
{
}

impl<T> InsertTotpSecretQuery for T where
    T: for<'a> Service<
        &'a InsertTotpSecretParams,
        Response = CreateTotpSecretResponse,
        Error = BoxedError,
    >
{
}
