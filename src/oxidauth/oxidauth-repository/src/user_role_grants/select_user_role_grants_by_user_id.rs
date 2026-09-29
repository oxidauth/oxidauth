pub use oxidauth_kernel::user_role_grants::{
    list_user_role_grants_by_user_id::ListUserRoleGrantsByUserId, UserRole,
};

use crate::prelude::*;

pub trait SelectUserRoleGrantsByUserIdQuery:
    for<'a> Service<
    &'a ListUserRoleGrantsByUserId,
    Response = Vec<UserRole>,
    Error = BoxedError,
>
{
}

impl<T> SelectUserRoleGrantsByUserIdQuery for T where
    T: for<'a> Service<
        &'a ListUserRoleGrantsByUserId,
        Response = Vec<UserRole>,
        Error = BoxedError,
    >
{
}
