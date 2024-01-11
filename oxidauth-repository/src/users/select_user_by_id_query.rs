pub use oxidauth_kernel::users::find_user_by_id::FindUserById;
pub use oxidauth_kernel::users::User;

use crate::prelude::*;

pub trait SelectUserByIdQuery:
    for<'a> Service<&'a FindUserById, Response = User, Error = BoxedError>
{
}

impl<T> SelectUserByIdQuery for T where
    T: for<'a> Service<&'a FindUserById, Response = User, Error = BoxedError>
{
}
