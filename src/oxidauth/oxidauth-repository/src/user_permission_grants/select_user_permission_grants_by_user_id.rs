pub use oxidauth_kernel::user_permission_grants::{
    list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserId,
    UserPermission,
};

use crate::prelude::*;

pub trait SelectUserPermissionGrantsByUserIdQuery:
    for<'a> Service<
    &'a ListUserPermissionGrantsByUserId,
    Response = Vec<UserPermission>,
    Error = BoxedError,
>
{
}

impl<T> SelectUserPermissionGrantsByUserIdQuery for T where
    T: for<'a> Service<
        &'a ListUserPermissionGrantsByUserId,
        Response = Vec<UserPermission>,
        Error = BoxedError,
    >
{
}
