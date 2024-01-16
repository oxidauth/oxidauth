use oxidauth_kernel::permissions::create_permission::CreatePermission;
pub use oxidauth_kernel::{permissions::Permission, service::Service};

pub use crate::prelude::*;

pub trait InsertPermissionQuery:
    for<'a> Service<&'a CreatePermission, Response = Permission, Error = BoxedError>
{
}

impl<T> InsertPermissionQuery for T where
    T: for<'a> Service<
        &'a CreatePermission,
        Response = Permission,
        Error = BoxedError,
    >
{
}
