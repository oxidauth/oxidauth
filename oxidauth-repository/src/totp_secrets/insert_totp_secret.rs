pub use oxidauth_kernel::service::Service;
use oxidauth_kernel::totp_secrets::create_totp_secret::CreateTotpSecretResponse;

pub use crate::prelude::*;

pub struct InsertTotpSecretParams {
    pub user_id: Uuid,
    pub secret_key: String,
}

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
