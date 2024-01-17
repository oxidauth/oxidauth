use oxidauth_kernel::role_role_grants::list_role_role_grants_by_parent_id::*;
pub use oxidauth_kernel::{service::Service, roles::Role};

pub use crate::prelude::*;

pub trait SelectRoleRoleGrantsByParentIdQuery:
    for<'a> Service<&'a ListRoleRoleGrantsByParentId, Response = Vec<RoleRoleGrantDetail>, Error = BoxedError>
{
}

impl<T> SelectRoleRoleGrantsByParentIdQuery for T where
    T: for<'a> Service<&'a ListRoleRoleGrantsByParentId, Response = Vec<RoleRoleGrantDetail>, Error = BoxedError>
{
}
