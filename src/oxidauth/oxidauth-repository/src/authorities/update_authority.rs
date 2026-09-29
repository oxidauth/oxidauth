pub use oxidauth_kernel::authorities::update_authority::UpdateAuthority;
pub use oxidauth_kernel::{authorities::Authority, service::Service};

pub use crate::prelude::*;

pub trait UpdateAuthorityQuery:
    for<'a> Service<&'a UpdateAuthority, Response = Authority, Error = BoxedError>
{
}

impl<T> UpdateAuthorityQuery for T where
    T: for<'a> Service<
        &'a UpdateAuthority,
        Response = Authority,
        Error = BoxedError,
    >
{
}
