use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{find_invitation::FindInvitationParams, Invitation},
    service::Service,
};

#[async_trait]
pub trait SelectInvitationByIdQuery:
    for<'a> Service<
    &'a FindInvitationParams,
    Response = Invitation,
    Error = BoxedError,
>
{
}

impl<T> SelectInvitationByIdQuery for T where
    T: for<'a> Service<
        &'a FindInvitationParams,
        Response = Invitation,
        Error = BoxedError,
    >
{
}
