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
    users::{
        update_user::{UpdateUser, UpdateUserService},
        User, UserStatus,
    },
};
use oxidauth_repository::invitations::delete_invitation_by_id::DeleteInvitationByIdQuery;

pub struct AcceptInvitationUseCase<I>
where
    I: DeleteInvitationByIdQuery,
{
    update_user: UpdateUserService,
    user_authority: CreateUserAuthorityService,
    delete_invitation: I,
}

impl<I> AcceptInvitationUseCase<I>
where
    I: DeleteInvitationByIdQuery,
{
    pub fn new(
        update_user: UpdateUserService,
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
impl<'a, I> Service<&'a AcceptInvitationParams> for AcceptInvitationUseCase<I>
where
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

        let Invitation { user_id, .. } = self
            .delete_invitation
            .call(&delete_invitation)
            .await?;

        let create_user_authority = CreateUserAuthorityParams {
            user_id,
            strategy: params.user_authority.strategy,
            params: params
                .user_authority
                .params
                .clone(),
        };

        let _user_authority = self
            .user_authority
            .call(&create_user_authority)
            .await?;

        let mut update_user: UpdateUser = (user_id, &params.user).into();

        let user = self
            .update_user
            .call(&mut update_user)
            .await?;

        Ok(user)
    }
}
