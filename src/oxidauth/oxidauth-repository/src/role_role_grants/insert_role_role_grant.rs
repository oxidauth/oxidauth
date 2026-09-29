use oxidauth_kernel::role_role_grants::create_role_role_grant::*;
pub use oxidauth_kernel::{roles::Role, service::Service};

pub use crate::prelude::*;

pub trait InsertRoleRoleGrantQuery:
    for<'a> Service<
    &'a CreateRoleRoleGrant,
    Response = RoleRoleGrant,
    Error = BoxedError,
>
{
}

impl<T> InsertRoleRoleGrantQuery for T where
    T: for<'a> Service<
        &'a CreateRoleRoleGrant,
        Response = RoleRoleGrant,
        Error = BoxedError,
    >
{
}
