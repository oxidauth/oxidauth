use std::error::Error;

use oxidauth_kernel::user_role_grants::{
    delete_user_role_grant::DeleteUserRoleGrant, UserRoleGrant,
};
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

pub trait DeleteUserRoleGrantQuery:
    for<'a> Service<
    &'a DeleteUserRoleGrant,
    Response = UserRoleGrant,
    Error = BoxedError,
>
{
}

impl<T> DeleteUserRoleGrantQuery for T where
    T: for<'a> Service<
        &'a DeleteUserRoleGrant,
        Response = UserRoleGrant,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct DeleteUserRoleGrantError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
