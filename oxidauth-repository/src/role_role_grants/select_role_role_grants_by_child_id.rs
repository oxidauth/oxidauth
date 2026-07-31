use oxidauth_kernel::role_role_grants::list_role_role_grants_by_child_id::*;
pub use oxidauth_kernel::{roles::Role, service::Service};

pub use crate::prelude::*;

pub trait SelectRoleRoleGrantsByChildIdQuery:
    for<'a> Service<
    &'a ListRoleRoleGrantsByChildId,
    Response = Vec<RoleRoleGrantDetail>,
    Error = BoxedError,
>
{
}

impl<T> SelectRoleRoleGrantsByChildIdQuery for T where
    T: for<'a> Service<
        &'a ListRoleRoleGrantsByChildId,
        Response = Vec<RoleRoleGrantDetail>,
        Error = BoxedError,
    >
{
}
