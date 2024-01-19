pub use oxidauth_kernel::public_keys::find_public_key_by_id::FindPublicKeyById;
pub use oxidauth_kernel::public_keys::PublicKey;

use crate::prelude::*;

pub trait SelectPublicKeyByIdQuery:
    for<'a> Service<&'a FindPublicKeyById, Response = PublicKey, Error = BoxedError>
{
}

impl<T> SelectPublicKeyByIdQuery for T where
    T: for<'a> Service<
        &'a FindPublicKeyById,
        Response = PublicKey,
        Error = BoxedError,
    >
{
}
