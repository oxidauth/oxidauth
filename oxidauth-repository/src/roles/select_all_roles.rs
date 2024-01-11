use oxidauth_kernel::roles::list_all_roles::ListAllRoles;
pub use oxidauth_kernel::{service::Service, roles::Role};

pub use crate::prelude::*;

pub trait SelectAllRolesQuery:
    for<'a> Service<&'a ListAllRoles, Response = Vec<Role>, Error = BoxedError>
{
}

impl<T> SelectAllRolesQuery for T where
    T: for<'a> Service<&'a ListAllRoles, Response = Vec<Role>, Error = BoxedError>
{
}

