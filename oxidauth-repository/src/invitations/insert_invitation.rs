use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{create_invitation::CreateInvitationParams, Invitation},
    service::Service,
};

#[async_trait]
pub trait InsertInvitationQuery:
    for<'a> Service<
    &'a CreateInvitationParams,
    Response = Invitation,
    Error = BoxedError,
>
{
}

impl<T> InsertInvitationQuery for T where
    T: for<'a> Service<
        &'a CreateInvitationParams,
        Response = Invitation,
        Error = BoxedError,
    >
{
}
