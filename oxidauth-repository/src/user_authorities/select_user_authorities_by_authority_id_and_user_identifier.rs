use oxidauth_kernel::{error::BoxedError, service::Service};

pub trait SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery:
    for<'a> Service<
    &'a SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams,
    Response = (),
    Error = BoxedError,
>
{
}

impl<T> SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery for T where
    T: for<'a> Service<
        &'a SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams,
        Response = (),
        Error = BoxedError,
    >
{
}

pub struct SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams {}
