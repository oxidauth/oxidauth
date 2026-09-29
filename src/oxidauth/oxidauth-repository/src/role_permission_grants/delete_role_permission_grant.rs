pub use oxidauth_kernel::role_permission_grants::RolePermissionGrant;
pub use oxidauth_kernel::service::Service;

pub use crate::prelude::*;

pub trait DeleteRolePermissionGrantQuery:
    for<'a> Service<
    &'a DeleteRolePermissionGrantParams,
    Response = RolePermissionGrant,
    Error = BoxedError,
>
{
}

impl<T> DeleteRolePermissionGrantQuery for T where
    T: for<'a> Service<
        &'a DeleteRolePermissionGrantParams,
        Response = RolePermissionGrant,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct DeleteRolePermissionGrantParams {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}
