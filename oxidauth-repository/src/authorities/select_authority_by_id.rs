pub use oxidauth_kernel::authorities::find_authority_by_id::FindAuthorityById;
pub use oxidauth_kernel::{service::Service, authorities::Authority};

pub use crate::prelude::*;

pub trait SelectAuthorityByIdQuery:
    for<'a> Service<&'a FindAuthorityById, Response = Authority, Error = BoxedError>
{
}

impl<T> SelectAuthorityByIdQuery for T where
    T: for<'a> Service<&'a FindAuthorityById, Response = Authority, Error = BoxedError>
{
}
