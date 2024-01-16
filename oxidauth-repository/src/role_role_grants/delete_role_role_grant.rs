use oxidauth_kernel::role_role_grants::delete_role_role_grant::*;
pub use oxidauth_kernel::{service::Service, roles::Role};

pub use crate::prelude::*;

pub trait DeleteRoleRoleGrantQuery:
    for<'a> Service<&'a DeleteRoleRoleGrant, Response = RoleRoleGrant, Error = BoxedError>
{
}

impl<T> DeleteRoleRoleGrantQuery for T where
    T: for<'a> Service<&'a DeleteRoleRoleGrant, Response = RoleRoleGrant, Error = BoxedError>
{
}
