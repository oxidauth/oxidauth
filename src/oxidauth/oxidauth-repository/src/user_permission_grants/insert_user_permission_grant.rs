use std::error::Error;

use oxidauth_kernel::user_permission_grants::{
    create_user_permission_grant::CreateUserPermissionGrant,
    UserPermissionGrant,
};
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

pub trait InsertUserPermissionGrantQuery:
    for<'a> Service<
    &'a CreateUserPermissionGrant,
    Response = UserPermissionGrant,
    Error = BoxedError,
>
{
}

impl<T> InsertUserPermissionGrantQuery for T where
    T: for<'a> Service<
        &'a CreateUserPermissionGrant,
        Response = UserPermissionGrant,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct InsertUserPermissionGrantError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
