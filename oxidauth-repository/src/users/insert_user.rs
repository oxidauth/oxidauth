use std::sync::Arc;
use std::error::Error;

use oxidauth_kernel::{users::create_user::CreateUser, error::BoxedError};
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

// pub type InsertUserQuery = Arc<
//     dyn for<'a> Service<
//         &'a CreateUser,
//         Response = User,
//         Error = BoxedError,
//     >,
// >;
pub type InsertUserQuery =
    dyn for<'a> Service<
        &'a CreateUser,
        Response = User,
        Error = BoxedError,
    >;

#[derive(Debug)]
pub struct InsertUserError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
