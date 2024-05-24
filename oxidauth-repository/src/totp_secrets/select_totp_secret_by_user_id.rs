pub use oxidauth_kernel::service::Service;
use oxidauth_kernel::totp_secrets::{
    select_totp_secret_by_user_id::SelectTOTPSecretByUserId, TOTPSecret,
};

pub use crate::prelude::*;

pub trait SelectTOTPSecrețByUserIdQuery:
    for<'a> Service<
    &'a SelectTOTPSecretByUserId,
    Response = TOTPSecret,
    Error = BoxedError,
>
{
}

impl<T> SelectTOTPSecrețByUserIdQuery for T where
    T: for<'a> Service<
        &'a SelectTOTPSecretByUserId,
        Response = TOTPSecret,
        Error = BoxedError,
    >
{
}
