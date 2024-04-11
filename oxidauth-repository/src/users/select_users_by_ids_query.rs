pub use oxidauth_kernel::users::find_users_by_ids::FindUsersByIds;
use oxidauth_kernel::users::find_users_by_ids::UsersByIds;

use crate::prelude::*;

pub trait SelectUsersByIdsQuery:
    for<'a> Service<&'a FindUsersByIds, Response = UsersByIds, Error = BoxedError>
{
}

impl<T> SelectUsersByIdsQuery for T where
    T: for<'a> Service<
        &'a FindUsersByIds,
        Response = UsersByIds,
        Error = BoxedError,
    >
{
}
