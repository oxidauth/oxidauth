pub use oxidauth_kernel::users::User;

use crate::prelude::*;

pub trait SelectUserByIdQuery:
    for<'a> Service<Uuid, Response = User, Error = BoxedError>
{
}

impl<T> SelectUserByIdQuery for T where
    T: for<'a> Service<Uuid, Response = User, Error = BoxedError>
{
}

// #[derive(Debug)]
// pub struct QueryUserByIdError {
//
// }
