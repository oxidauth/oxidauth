use std::error::Error;

use oxidauth_kernel::user_permission_grants::{
    delete_user_permission_grant::DeleteUserPermissionGrant,
    UserPermissionGrant,
};
pub use oxidauth_kernel::{service::Service, users::User};

pub use crate::prelude::*;

pub trait DeleteUserPermissionGrantQuery:
    for<'a> Service<
    &'a DeleteUserPermissionGrant,
    Response = UserPermissionGrant,
    Error = BoxedError,
>
{
}

impl<T> DeleteUserPermissionGrantQuery for T where
    T: for<'a> Service<
        &'a DeleteUserPermissionGrant,
        Response = UserPermissionGrant,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct DeleteUserPermissionGrantError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
