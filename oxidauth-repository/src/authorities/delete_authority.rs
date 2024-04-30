pub use oxidauth_kernel::authorities::delete_authority::DeleteAuthority;
pub use oxidauth_kernel::{authorities::Authority, service::Service};

pub use crate::prelude::*;

pub trait DeleteAuthorityQuery:
    for<'a> Service<&'a DeleteAuthority, Response = Authority, Error = BoxedError>
{
}

impl<T> DeleteAuthorityQuery for T where
    T: for<'a> Service<
        &'a DeleteAuthority,
        Response = Authority,
        Error = BoxedError,
    >
{
}
