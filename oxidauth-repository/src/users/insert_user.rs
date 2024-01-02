use std::error::Error;

use oxidauth_kernel::users::user_create::UserCreate;
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

pub trait InsertUserRepo:
    for<'a> Service<&'a InsertUserParams, Response = User, Error = InsertUserError>
{
}

// #[derive(Debug)]
// pub struct InsertUserParams {
//     pub id: Option<Uuid>,
//     pub username: String,
//     pub email: Option<String>,
//     pub first_name: Option<String>,
//     pub last_name: Option<String>,
//     pub profile: Option<Value>,
//     pub kind: Option<String>,
//     pub status: Option<String>,
// }

pub type InsertUserParams = UserCreate;

#[derive(Debug)]
pub struct InsertUserError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
