use std::ops::Add;

use async_trait::async_trait;
use chrono::{Days, Utc};
use oxidauth_kernel::{
    error::BoxedError,
    invitations::create_invitation::{
        CreateInvitationParams, CreateInvitationResponse,
    },
    service::Service,
    users::create_user::CreateUserTrait,
};
use oxidauth_repository::invitations::insert_invitation::{
    InsertInvitationParams, InsertInvitationQuery,
};

pub struct CreateInvitationUseCase<T, U>
where
    T: InsertInvitationQuery,
    U: CreateUserTrait,
{
    insert_invitation: T,
    create_user: U,
}

impl<T, U> CreateInvitationUseCase<T, U>
where
    T: InsertInvitationQuery,
    U: CreateUserTrait,
{
    pub fn new(insert_invitation: T, create_user: U) -> Self {
        Self {
            insert_invitation,
            create_user,
        }
    }
}

#[async_trait]
impl<'a, T, U> Service<&'a CreateInvitationParams>
    for CreateInvitationUseCase<T, U>
where
    T: InsertInvitationQuery,
    U: CreateUserTrait,
{
    type Response = CreateInvitationResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_invitation_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a CreateInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        let user = self
            .create_user
            .call(&params.user)
            .await?;

        let insert_invitation_params = InsertInvitationParams {
            id: params.id,
            user_id: user.id,
            // TODO(dewey4iv): https://www.pivotaltracker.com/story/show/186949366
            expires_at: params
                .expires_at
                .unwrap_or_else(|| Utc::now().add(Days::new(7))),
        };

        let invitation = self
            .insert_invitation
            .call(&insert_invitation_params)
            .await?;

        Ok(CreateInvitationResponse { invitation, user })
    }
}
