pub use oxidauth_kernel::users::delete_user_by_id::DeleteUserById;
pub use oxidauth_kernel::users::User;

use crate::prelude::*;

pub trait DeleteUserByIdQuery:
    for<'a> Service<&'a DeleteUserById, Response = User, Error = BoxedError>
{
}

impl<T> DeleteUserByIdQuery for T where
    T: for<'a> Service<&'a DeleteUserById, Response = User, Error = BoxedError>
{
}
