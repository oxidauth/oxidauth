pub use oxidauth_kernel::authorities::list_all_authorities::ListAllAuthorities;
pub use oxidauth_kernel::{authorities::Authority, service::Service};

pub use crate::prelude::*;

pub trait SelectAllAuthoritiesQuery:
    for<'a> Service<
    &'a ListAllAuthorities,
    Response = Vec<Authority>,
    Error = BoxedError,
>
{
}

impl<T> SelectAllAuthoritiesQuery for T where
    T: for<'a> Service<
        &'a ListAllAuthorities,
        Response = Vec<Authority>,
        Error = BoxedError,
    >
{
}
