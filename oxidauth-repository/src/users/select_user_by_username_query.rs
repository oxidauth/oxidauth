pub use oxidauth_kernel::users::User;

use crate::prelude::*;

pub trait SelectUserByUsernameQuery:
    for<'a> Service<String, Response = User, Error = BoxedError>
{
}

impl<T> SelectUserByUsernameQuery for T where
    T: for<'a> Service<String, Response = User, Error = BoxedError>
{
}
