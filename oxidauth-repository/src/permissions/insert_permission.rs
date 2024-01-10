use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::{service::Service, permissions::Permission};

pub use crate::prelude::*;

pub trait InsertPermissionQuery:
    for<'a> Service<&'a str, Response = Permission, Error = BoxedError>
{
}

impl<T> InsertPermissionQuery for T where
    T: for<'a> Service<&'a str, Response = Permission, Error = BoxedError>
{
}

