pub use oxidauth_kernel::users::{list_all_users::ListAllUsers, User};

use crate::prelude::*;

pub trait SelectAllUsersQuery:
    for<'a> Service<&'a ListAllUsers, Response = Vec<User>, Error = BoxedError>
{
}

impl<T> SelectAllUsersQuery for T where
    T: for<'a> Service<
        &'a ListAllUsers,
        Response = Vec<User>,
        Error = BoxedError,
    >
{
}
