use oxidauth_kernel::permissions::list_all_permissions::*;
pub use oxidauth_kernel::{permissions::Permission, service::Service};

pub use crate::prelude::*;

pub trait SelectAllPermissionsQuery:
    for<'a> Service<
    &'a ListAllPermissions,
    Response = Vec<Permission>,
    Error = BoxedError,
>
{
}

impl<T> SelectAllPermissionsQuery for T where
    T: for<'a> Service<
        &'a ListAllPermissions,
        Response = Vec<Permission>,
        Error = BoxedError,
    >
{
}
