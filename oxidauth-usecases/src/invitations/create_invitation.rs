use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::create_invitation::{
        CreateInvitationParams, CreateInvitationResponse,
    },
    service::Service,
};
use oxidauth_repository::{
    invitations::insert_invitation::{
        InsertInvitationParams, InsertInvitationQuery,
    },
    users::insert_user::InsertUserQuery,
};

pub struct CreateInvitationUseCase<T, U>
where
    T: InsertInvitationQuery,
    U: InsertUserQuery,
{
    insert_invitation: T,
    insert_user: U,
}

impl<T, U> CreateInvitationUseCase<T, U>
where
    T: InsertInvitationQuery,
    U: InsertUserQuery,
{
    pub fn new(insert_invitation: T, insert_user: U) -> Self {
        Self {
            insert_invitation,
            insert_user,
        }
    }
}

#[async_trait]
impl<'a, T, U> Service<&'a CreateInvitationParams>
    for CreateInvitationUseCase<T, U>
where
    T: InsertInvitationQuery,
    U: InsertUserQuery,
{
    type Response = CreateInvitationResponse;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a CreateInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        let user = self
            .insert_user
            .call(&params.user)
            .await?;

        let insert_invitation_params = InsertInvitationParams {
            id: params.id,
            user_id: user.id,
            expires_at: params.expires_at,
        };

        let invitation = self
            .insert_invitation
            .call(&insert_invitation_params)
            .await?;

        Ok(CreateInvitationResponse { invitation, user })
    }
}
