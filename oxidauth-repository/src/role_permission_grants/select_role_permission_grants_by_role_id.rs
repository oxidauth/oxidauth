use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::*;
pub use oxidauth_kernel::service::Service;

pub use crate::prelude::*;

pub trait SelectRolePermissionGrantsByRoleIdQuery:
    for<'a> Service<&'a ListRolePermissionGrantsByRoleId, Response = Vec<RolePermission>, Error = BoxedError>
{
}

impl<T> SelectRolePermissionGrantsByRoleIdQuery for T where
    T: for<'a> Service<&'a ListRolePermissionGrantsByRoleId, Response = Vec<RolePermission>, Error = BoxedError>
{
}
