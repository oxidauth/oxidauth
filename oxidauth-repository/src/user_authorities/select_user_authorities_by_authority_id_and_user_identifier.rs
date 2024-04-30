use uuid::Uuid;

use oxidauth_kernel::{
    error::BoxedError, service::Service, user_authorities::UserAuthority,
};

pub trait SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery:
    for<'a> Service<
    &'a SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams,
    Response = UserAuthority,
    Error = BoxedError,
>
{
}

impl<T> SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery for T where
    T: for<'a> Service<
        &'a SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams,
        Response = UserAuthority,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams {
    pub authority_id: Uuid,
    pub user_identifier: String,
}
