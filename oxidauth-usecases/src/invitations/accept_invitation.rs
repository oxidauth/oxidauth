use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{
        accept_invitation::{AcceptInvitationParams, AcceptInvitationTrait},
        delete_invitation::DeleteInvitationParams, Invitation,
    },
    user_authorities::create_user_authority::{
        CreateUserAuthorityParams, CreateUserAuthorityService,
    },
    users::{
        update_user::{UpdateUser, UpdateUserService},
        User,
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
impl<I> AcceptInvitationTrait for AcceptInvitationUseCase<I>
where
    I: DeleteInvitationByIdQuery,
{
    #[tracing::instrument(name = "accept_invitation_usecase", skip(self))]
    async fn accept_invitation(
        &self,
        params: &AcceptInvitationParams,
    ) -> Result<User, BoxedError> {
        let delete_invitation = DeleteInvitationParams {
            id: params.invitation_id,
        };

        let Invitation { user_id, .. } = self
            .delete_invitation
            .call(&delete_invitation)
            .await?;

        let create_user_authority = CreateUserAuthorityParams {
            user_id,
            client_key: params
                .user_authority
                .client_key,
            params: params
                .user_authority
                .params
                .clone(),
        };

        let _user_authority = self
            .user_authority
            .create_user_authority(&create_user_authority)
            .await?;

        let mut update_user: UpdateUser = (user_id, &params.user).into();

        let user = self
            .update_user
            .update_user(&mut update_user)
            .await?;

        Ok(user)
    }
}
