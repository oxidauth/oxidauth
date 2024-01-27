use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{create_invitation::CreateInvitationParams, Invitation},
    service::Service,
};
use oxidauth_repository::invitations::insert_invitation::InsertInvitationQuery;

pub struct CreateInvitationUseCase<T>
where
    T: InsertInvitationQuery,
{
    insert_invitation: T,
}

impl<T> CreateInvitationUseCase<T>
where
    T: InsertInvitationQuery,
{
    pub fn new(insert_invitation: T) -> Self {
        Self { insert_invitation }
    }
}

#[async_trait]
impl<'a, T> Service<&'a CreateInvitationParams> for CreateInvitationUseCase<T>
where
    T: InsertInvitationQuery,
{
    type Response = Invitation;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a CreateInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        self.insert_invitation
            .call(params)
            .await
    }
}
