use oxidauth_kernel::permissions::find_permission_by_parts::*;
pub use oxidauth_kernel::{permissions::Permission, service::Service};

pub use crate::prelude::*;

pub trait SelectPermissionByPartsQuery:
    for<'a> Service<
    &'a FindPermissionByParts,
    Response = Permission,
    Error = BoxedError,
>
{
}

impl<T> SelectPermissionByPartsQuery for T where
    T: for<'a> Service<
        &'a FindPermissionByParts,
        Response = Permission,
        Error = BoxedError,
    >
{
}
