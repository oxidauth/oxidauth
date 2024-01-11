use oxidauth_kernel::roles::delete_role::DeleteRole;
pub use oxidauth_kernel::{service::Service, roles::Role};

pub use crate::prelude::*;

pub trait DeleteRoleQuery:
    for<'a> Service<&'a DeleteRole, Response = Role, Error = BoxedError>
{
}

impl<T> DeleteRoleQuery for T where
    T: for<'a> Service<&'a DeleteRole, Response = Role, Error = BoxedError>
{
}

