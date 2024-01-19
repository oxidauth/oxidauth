pub use oxidauth_kernel::authorities::create_authority::CreateAuthority;
pub use oxidauth_kernel::{service::Service, authorities::Authority};

pub use crate::prelude::*;

pub trait InsertAuthorityQuery:
    for<'a> Service<&'a CreateAuthority, Response = Authority, Error = BoxedError>
{
}

impl<T> InsertAuthorityQuery for T where
    T: for<'a> Service<&'a CreateAuthority, Response = Authority, Error = BoxedError>
{
}
