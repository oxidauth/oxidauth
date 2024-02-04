use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{
        accept_invitation::AcceptInvitationParams,
        delete_invitation::DeleteInvitationParams, Invitation,
    },
    service::Service,
    user_authorities::create_user_authority::{
        CreateUserAuthorityParams, CreateUserAuthorityService,
    },
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
    user_authority: CreateUserAuthorityService,
    delete_invitation: I,
}

impl<U, I> AcceptInvitationUseCase<U, I>
where
    U: UpdateUserQuery,
    I: DeleteInvitationByIdQuery,
{
    pub fn new(
        update_user: U,
        user_authority: CreateUserAuthorityService,
        delete_invitation: I,
    ) -> Self {
        Self {
            update_user,
            user_authority,
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

    #[tracing::instrument(name = "accept_invitation_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a AcceptInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        let delete_invitation = DeleteInvitationParams {
            id: params.invitation_id,
        };

        // let Invitation { user_id, .. } = self
        //     .delete_invitation
        //     .call(&delete_invitation)
        //     .await?;

        let invitation = self
            .delete_invitation
            .call(&delete_invitation)
            .await;

        dbg!(&invitation);

        let Invitation { user_id, .. } = invitation?;

        let create_user_authority = CreateUserAuthorityParams {
            user_id,
            strategy: params.user_authority.strategy,
            params: params
                .user_authority
                .params
                .clone(),
        };

        let user_authority = self
            .user_authority
            .call(&create_user_authority)
            .await;

        dbg!(&user_authority);

        user_authority?;

        let update_user = (user_id, &params.user).into();

        let user = self
            .update_user
            .call(&update_user)
            .await;

        dbg!(&user);

        let user = user?;

        Ok(user)
    }
}
