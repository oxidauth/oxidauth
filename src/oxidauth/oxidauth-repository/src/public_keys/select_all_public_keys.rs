pub use oxidauth_kernel::public_keys::list_all_public_keys::ListAllPublicKeys;
pub use oxidauth_kernel::public_keys::PublicKey;

use crate::prelude::*;

pub trait SelectAllPublicKeysQuery:
    for<'a> Service<
    &'a ListAllPublicKeys,
    Response = Vec<PublicKey>,
    Error = BoxedError,
>
{
}

impl<T> SelectAllPublicKeysQuery for T where
    T: for<'a> Service<
        &'a ListAllPublicKeys,
        Response = Vec<PublicKey>,
        Error = BoxedError,
    >
{
}
