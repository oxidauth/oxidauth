use std::error::Error;

pub use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::update_user_authority::UpdateUserAuthority,
};
pub use oxidauth_kernel::{service::Service, user_authorities::UserAuthority};

pub use crate::prelude::*;

pub trait UpdateUserAuthorityQuery:
    for<'a> Service<
    &'a UpdateUserAuthority,
    Response = UserAuthority,
    Error = BoxedError,
>
{
}

impl<T> UpdateUserAuthorityQuery for T where
    T: for<'a> Service<
        &'a UpdateUserAuthority,
        Response = UserAuthority,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct UpdateUserAuthorityError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
