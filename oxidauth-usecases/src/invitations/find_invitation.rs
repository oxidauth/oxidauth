use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{find_invitation::FindInvitationParams, Invitation},
    service::Service,
};
use oxidauth_repository::invitations::select_invitation_by_id::SelectInvitationByIdQuery;

pub struct FindInvitationUseCase<T>
where
    T: SelectInvitationByIdQuery,
{
    select_invitation_by_id: T,
}

impl<T> FindInvitationUseCase<T>
where
    T: SelectInvitationByIdQuery,
{
    pub fn new(select_invitation_by_id: T) -> Self {
        Self {
            select_invitation_by_id,
        }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindInvitationParams> for FindInvitationUseCase<T>
where
    T: SelectInvitationByIdQuery,
{
    type Response = Invitation;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a FindInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        self.select_invitation_by_id
            .call(params)
            .await
    }
}
