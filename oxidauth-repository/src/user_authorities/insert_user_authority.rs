use oxidauth_kernel::user_authorities::create_user_authority::CreateUserAuthority;
pub use oxidauth_kernel::{service::Service, user_authorities::UserAuthority};

pub use crate::prelude::*;

pub trait InsertUserAuthorityQuery:
    for<'a> Service<
    &'a CreateUserAuthority,
    Response = UserAuthority,
    Error = BoxedError,
>
{
}

impl<T> InsertUserAuthorityQuery for T where
    T: for<'a> Service<
        &'a CreateUserAuthority,
        Response = UserAuthority,
        Error = BoxedError,
    >
{
}

#[derive(Debug)]
pub struct InsertUserAuthority {
    pub user_id: Option<Uuid>,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
}
