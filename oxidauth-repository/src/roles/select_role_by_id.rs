use oxidauth_kernel::roles::find_role_by_id::FindRoleById;
pub use oxidauth_kernel::{roles::Role, service::Service};

pub use crate::prelude::*;

pub trait SelectRoleByIdQuery:
    for<'a> Service<&'a FindRoleById, Response = Role, Error = BoxedError>
{
}

impl<T> SelectRoleByIdQuery for T where
    T: for<'a> Service<&'a FindRoleById, Response = Role, Error = BoxedError>
{
}
