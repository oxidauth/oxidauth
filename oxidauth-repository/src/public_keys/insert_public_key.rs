use oxidauth_kernel::public_keys::PublicKey;

use crate::prelude::*;

pub trait InsertPublicKeyQuery:
    for<'a> Service<
    &'a InsertPublicKeyParams,
    Response = PublicKey,
    Error = BoxedError,
>
{
}

impl<T> InsertPublicKeyQuery for T where
    T: for<'a> Service<
        &'a InsertPublicKeyParams,
        Response = PublicKey,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct InsertPublicKeyParams {
    pub id: Option<Uuid>,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}
