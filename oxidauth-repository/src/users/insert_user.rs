use std::error::Error;

use oxidauth_kernel::users::create_user::CreateUser;
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

pub trait InsertUserQuery:
    for<'a> Service<&'a CreateUser, Response = User, Error = BoxedError>
{
}

impl<T> InsertUserQuery for T where
    T: for<'a> Service<&'a CreateUser, Response = User, Error = BoxedError>
{
}

#[derive(Debug)]
pub struct InsertUserError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
