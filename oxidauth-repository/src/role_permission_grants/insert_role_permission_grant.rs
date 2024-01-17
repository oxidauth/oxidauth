use oxidauth_kernel::role_permission_grants::create_role_permission_grant::*;
pub use oxidauth_kernel::service::Service;

pub use crate::prelude::*;

pub trait InsertRolePermissionGrantQuery:
    for<'a> Service<&'a InsertRolePermissionGrant, Response = RolePermissionGrant, Error = BoxedError>
{
}

impl<T> InsertRolePermissionGrantQuery for T where
    T: for<'a> Service<&'a InsertRolePermissionGrant, Response = RolePermissionGrant, Error = BoxedError>
{
}

#[derive(Debug)]
pub struct InsertRolePermissionGrant {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}
