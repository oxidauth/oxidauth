use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{
        accept_invitation::AcceptInvitationParams,
        delete_invitation::DeleteInvitationParams,
    },
    service::Service,
    users::User,
};
use oxidauth_repository::{
    invitations::delete_invitation_by_id::DeleteInvitationByIdQuery,
    users::update_user::UpdateUserQuery,
};

pub struct AcceptInvitationUseCase<U, I>
where
    U: UpdateUserQuery,
    I: DeleteInvitationByIdQuery,
{
    update_user: U,
    delete_invitation: I,
}

impl<U, I> AcceptInvitationUseCase<U, I>
where
    U: UpdateUserQuery,
    I: DeleteInvitationByIdQuery,
{
    pub fn new(delete_invitation: I, update_user: U) -> Self {
        Self {
            update_user,
            delete_invitation,
        }
    }
}

#[async_trait]
impl<'a, U, I> Service<&'a AcceptInvitationParams>
    for AcceptInvitationUseCase<U, I>
where
    U: UpdateUserQuery,
    I: DeleteInvitationByIdQuery,
{
    type Response = User;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a AcceptInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        let delete_invitation = DeleteInvitationParams {
            id: params.invitation_id,
        };

        let invitation = self
            .delete_invitation
            .call(&delete_invitation)
            .await?;

        let update_user = (
            invitation.user_id,
            &params.user,
        )
            .into();

        let user = self
            .update_user
            .call(&update_user)
            .await?;

        Ok(user)
    }
}
