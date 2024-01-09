use std::error::Error;
use std::sync::Arc;

use oxidauth_kernel::{error::BoxedError, users::create_user::CreateUser};
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

// pub type InsertUserQuery = Arc<
//     dyn for<'a> Service<
//         &'a CreateUser,
//         Response = User,
//         Error = BoxedError,
//     >,
// >;
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
