use oxidauth_kernel::public_keys::{
    delete_public_key::DeletePublicKey, PublicKey,
};

use crate::prelude::*;

pub trait DeletePublicKeyQuery:
    for<'a> Service<&'a DeletePublicKey, Response = PublicKey, Error = BoxedError>
{
}

impl<T> DeletePublicKeyQuery for T where
    T: for<'a> Service<
        &'a DeletePublicKey,
        Response = PublicKey,
        Error = BoxedError,
    >
{
}
