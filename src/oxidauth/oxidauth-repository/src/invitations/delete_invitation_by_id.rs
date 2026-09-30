use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{Invitation, delete_invitation::DeleteInvitationParams},
};

#[async_trait]
pub trait DeleteInvitationByIdQuery: Send + Sync + 'static {
    async fn delete_invitation_by_id(
        &self,
        params: &DeleteInvitationParams,
    ) -> Result<Invitation, BoxedError>;
}
