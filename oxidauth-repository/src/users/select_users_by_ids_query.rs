pub use oxidauth_kernel::users::find_users_by_ids::FindUsersByIds;
pub use oxidauth_kernel::users::User;

use crate::prelude::*;

pub trait SelectUsersByIdsQuery:
    for<'a> Service<
    &'a FindUsersByIds,
    Response = (Vec<User>, Vec<Uuid>),
    Error = BoxedError,
>
{
}

impl<T> SelectUsersByIdsQuery for T where
    T: for<'a> Service<
        &'a FindUsersByIds,
        Response = (Vec<User>, Vec<Uuid>),
        Error = BoxedError,
    >
{
}
