pub use oxidauth_kernel::user_authorities::find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityId;
pub use oxidauth_kernel::user_authorities::UserAuthorityWithAuthority;

use crate::prelude::*;

pub trait SelectUserAuthorityByUserIdAndAuthorityIdQuery:
    for<'a> Service<
    &'a FindUserAuthorityByUserIdAndAuthorityId,
    Response = UserAuthorityWithAuthority,
    Error = BoxedError,
>
{
}

impl<T> SelectUserAuthorityByUserIdAndAuthorityIdQuery for T where
    T: for<'a> Service<
        &'a FindUserAuthorityByUserIdAndAuthorityId,
        Response = UserAuthorityWithAuthority,
        Error = BoxedError,
    >
{
}
