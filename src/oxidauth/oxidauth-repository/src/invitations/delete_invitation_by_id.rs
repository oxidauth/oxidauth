use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{delete_invitation::DeleteInvitationParams, Invitation},
    service::Service,
};

#[async_trait]
pub trait DeleteInvitationByIdQuery:
    for<'a> Service<
    &'a DeleteInvitationParams,
    Response = Invitation,
    Error = BoxedError,
>
{
}

impl<T> DeleteInvitationByIdQuery for T where
    T: for<'a> Service<
        &'a DeleteInvitationParams,
        Response = Invitation,
        Error = BoxedError,
    >
{
}
