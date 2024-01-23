pub use oxidauth_kernel::private_keys::PrivateKey;
use oxidauth_kernel::private_keys::find_most_recent_public_key::FindMostRecentPrivateKey;

use crate::prelude::*;

pub trait SelectMostRecentPrivateKeyQuery:
    for<'a> Service<
    &'a FindMostRecentPrivateKey,
    Response = PrivateKey,
    Error = BoxedError,
>
{
}

impl<T> SelectMostRecentPrivateKeyQuery for T where
    T: for<'a> Service<
        &'a FindMostRecentPrivateKey,
        Response = PrivateKey,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct SelectMostRecentPrivateKey {}
