use oxidauth_kernel::permissions::delete_permission::DeletePermission;
pub use oxidauth_kernel::{permissions::Permission, service::Service};

pub use crate::prelude::*;

pub trait DeletePermissionQuery:
    for<'a> Service<&'a DeletePermission, Response = Permission, Error = BoxedError>
{
}

impl<T> DeletePermissionQuery for T where
    T: for<'a> Service<
        &'a DeletePermission,
        Response = Permission,
        Error = BoxedError,
    >
{
}
