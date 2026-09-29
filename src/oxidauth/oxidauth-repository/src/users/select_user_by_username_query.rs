pub use oxidauth_kernel::users::{User, Username};

use crate::prelude::*;

pub trait SelectUserByUsernameQuery:
    for<'a> Service<&'a Username, Response = Option<User>, Error = BoxedError>
{
}

impl<T> SelectUserByUsernameQuery for T where
    T: for<'a> Service<
        &'a Username,
        Response = Option<User>,
        Error = BoxedError,
    >
{
}
