use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{
        find_invitation::{FindInvitationParams, FindInvitationTrait},
        Invitation,
    },
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
impl<T> FindInvitationTrait for FindInvitationUseCase<T>
where
    T: SelectInvitationByIdQuery,
{
    #[tracing::instrument(name = "find_invitation_usecase", skip(self))]
    async fn find_invitation(
        &self,
        params: &FindInvitationParams,
    ) -> Result<Invitation, BoxedError> {
        self.select_invitation_by_id
            .call(params)
            .await
    }
}
