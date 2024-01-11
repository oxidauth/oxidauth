use oxidauth_kernel::roles::create_role::CreateRole;
pub use oxidauth_kernel::{service::Service, roles::Role};

pub use crate::prelude::*;

pub trait InsertRoleQuery:
    for<'a> Service<&'a CreateRole, Response = Role, Error = BoxedError>
{
}

impl<T> InsertRoleQuery for T where
    T: for<'a> Service<&'a CreateRole, Response = Role, Error = BoxedError>
{
}
