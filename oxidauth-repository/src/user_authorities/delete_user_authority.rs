use std::error::Error;

use oxidauth_kernel::user_authorities::{
    delete_user_authority::DeleteUserAuthority, UserAuthority,
};
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

pub trait DeleteUserAuthorityQuery:
    for<'a> Service<
    &'a DeleteUserAuthority,
    Response = UserAuthority,
    Error = BoxedError,
>
{
}

impl<T> DeleteUserAuthorityQuery for T where
    T: for<'a> Service<
        &'a DeleteUserAuthority,
        Response = UserAuthority,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct DeleteUserAuthorityError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
