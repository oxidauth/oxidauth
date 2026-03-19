use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{
        delete_invitation::{DeleteInvitationParams, DeleteInvitationTrait},
        Invitation,
    },
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
impl<T> DeleteInvitationTrait for DeleteInvitationUseCase<T>
where
    T: DeleteInvitationByIdQuery,
{
    #[tracing::instrument(name = "delete_invitation_usecase", skip(self))]
    async fn delete_invitation(
        &self,
        params: &DeleteInvitationParams,
    ) -> Result<Invitation, BoxedError> {
        self.delete_invitation_by_id
            .call(params)
            .await
    }
}
