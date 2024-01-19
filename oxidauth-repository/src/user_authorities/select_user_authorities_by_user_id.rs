pub use oxidauth_kernel::user_authorities::list_user_authorities_by_user_id::ListUserAuthoritiesByUserId;
pub use oxidauth_kernel::user_authorities::UserAuthorityWithAuthority;

use crate::prelude::*;

pub trait SelectUserAuthoritiesByUserIdQuery:
    for<'a> Service<
    &'a ListUserAuthoritiesByUserId,
    Response = Vec<UserAuthorityWithAuthority>,
    Error = BoxedError,
>
{
}

impl<T> SelectUserAuthoritiesByUserIdQuery for T where
    T: for<'a> Service<
        &'a ListUserAuthoritiesByUserId,
        Response = Vec<UserAuthorityWithAuthority>,
        Error = BoxedError,
    >
{
}
