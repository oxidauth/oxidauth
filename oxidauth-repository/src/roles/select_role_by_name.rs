use oxidauth_kernel::roles::find_role_by_name::FindRoleByName;
pub use oxidauth_kernel::{roles::Role, service::Service};

pub use crate::prelude::*;

pub trait SelectRoleByNameQuery:
    for<'a> Service<&'a FindRoleByName, Response = Role, Error = BoxedError>
{
}

impl<T> SelectRoleByNameQuery for T where
    T: for<'a> Service<&'a FindRoleByName, Response = Role, Error = BoxedError>
{
}
