use oxidauth_kernel::roles::update_role::UpdateRole;
pub use oxidauth_kernel::{roles::Role, service::Service};

pub use crate::prelude::*;

pub trait UpdateRoleQuery:
    for<'a> Service<&'a UpdateRole, Response = Role, Error = BoxedError>
{
}

impl<T> UpdateRoleQuery for T where
    T: for<'a> Service<&'a UpdateRole, Response = Role, Error = BoxedError>
{
}
