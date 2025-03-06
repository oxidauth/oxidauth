use oxidauth_kernel::authorities::find_authority_by_client_key::FindAuthorityByClientKey;
pub use oxidauth_kernel::{authorities::Authority, service::Service};

pub use crate::prelude::*;

pub trait SelectAuthorityByClientKeyQuery:
    for<'a> Service<
        &'a FindAuthorityByClientKey,
        Response = Option<Authority>,
        Error = BoxedError,
    >
{
}

impl<T> SelectAuthorityByClientKeyQuery for T where
    T: for<'a> Service<
            &'a FindAuthorityByClientKey,
            Response = Option<Authority>,
            Error = BoxedError,
        >
{
}
