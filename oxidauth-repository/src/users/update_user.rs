use std::error::Error;

pub use oxidauth_kernel::{error::BoxedError, users::update_user::UpdateUser};
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

pub trait UpdateUserQuery:
    for<'a> Service<&'a UpdateUser, Response = User, Error = BoxedError>
{
}

impl<T> UpdateUserQuery for T where
    T: for<'a> Service<&'a UpdateUser, Response = User, Error = BoxedError>
{
}

#[derive(Debug)]
pub struct UpdateUserError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
