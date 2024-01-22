pub use oxidauth_kernel::private_keys::PrivateKey;

use crate::prelude::*;

pub trait SelectMostRecentPrivateKeyQuery:
    for<'a> Service<
    &'a SelectMostRecentPrivateKey,
    Response = PrivateKey,
    Error = BoxedError,
>
{
}

impl<T> SelectMostRecentPrivateKeyQuery for T where
    T: for<'a> Service<
        &'a SelectMostRecentPrivateKey,
        Response = PrivateKey,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct SelectMostRecentPrivateKey {}
