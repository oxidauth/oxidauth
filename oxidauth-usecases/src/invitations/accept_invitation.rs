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

pub struct AcceptInvitationUseCase<I, U>
where
    I: DeleteInvitationByIdQuery,
    U: UpdateUserQuery,
{
    delete_invitation: I,
    update_user: U,
}

impl<I, U> AcceptInvitationUseCase<I, U>
where
    I: DeleteInvitationByIdQuery,
    U: UpdateUserQuery,
{
    pub fn new(delete_invitation: I, update_user: U) -> Self {
        Self {
            delete_invitation,
            update_user,
        }
    }
}

#[async_trait]
impl<'a, I, U> Service<&'a AcceptInvitationParams>
    for AcceptInvitationUseCase<I, U>
where
    I: DeleteInvitationByIdQuery,
    U: UpdateUserQuery,
{
    type Response = User;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a AcceptInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        let update_user = (&params.user).into();

        let user = self
            .update_user
            .call(&update_user)
            .await?;

        let delete_invitation = DeleteInvitationParams {
            id: params.invitation_id,
        };

        self.delete_invitation
            .call(&delete_invitation)
            .await?;

        Ok(user)
    }
}
