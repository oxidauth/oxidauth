use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{delete_invitation::DeleteInvitationParams, Invitation},
    service::Service,
};
use oxidauth_repository::invitations::delete_invitation_by_id::DeleteInvitationByIdQuery;

pub struct DeleteInvitationUseCase<T>
where
    T: DeleteInvitationByIdQuery,
{
    delete_invitation_by_id: T,
}

impl<T> DeleteInvitationUseCase<T>
where
    T: DeleteInvitationByIdQuery,
{
    pub fn new(delete_invitation_by_id: T) -> Self {
        Self {
            delete_invitation_by_id,
        }
    }
}

#[async_trait]
impl<'a, T> Service<&'a DeleteInvitationParams> for DeleteInvitationUseCase<T>
where
    T: DeleteInvitationByIdQuery,
{
    type Response = Invitation;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a DeleteInvitationParams,
    ) -> Result<Self::Response, Self::Error> {
        self.delete_invitation_by_id
            .call(params)
            .await
    }
}
