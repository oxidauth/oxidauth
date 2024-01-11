use oxidauth_kernel::permissions::delete_permission::DeletePermission;
pub use oxidauth_kernel::{service::Service, permissions::Permission};

pub use crate::prelude::*;

pub trait DeletePermissionQuery:
    for<'a> Service<&'a DeletePermission, Response = Permission, Error = BoxedError>
{
}

impl<T> DeletePermissionQuery for T where
    T: for<'a> Service<&'a DeletePermission, Response = Permission, Error = BoxedError>
{
}
