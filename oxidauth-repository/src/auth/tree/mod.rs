pub use oxidauth_kernel::{auth::tree::*, service::Service};

pub use crate::prelude::*;

pub trait PermissionTreeQuery:
    for<'a> Service<
    &'a PermissionSearch,
    Response = PermissionsResponse,
    Error = BoxedError,
>
{
}

impl<T> PermissionTreeQuery for T where
    T: for<'a> Service<
        &'a PermissionSearch,
        Response = PermissionsResponse,
        Error = BoxedError,
    >
{
}
