use std::error::Error;

use oxidauth_kernel::user_role_grants::{
    create_user_role_grant::CreateUserRoleGrant, UserRoleGrant,
};
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

pub trait InsertUserRoleGrantQuery:
    for<'a> Service<
    &'a CreateUserRoleGrant,
    Response = UserRoleGrant,
    Error = BoxedError,
>
{
}

impl<T> InsertUserRoleGrantQuery for T where
    T: for<'a> Service<
        &'a CreateUserRoleGrant,
        Response = UserRoleGrant,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct InsertUserRoleGrantError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
