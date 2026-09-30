use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{Invitation, find_invitation::FindInvitationParams},
};

#[async_trait]
pub trait SelectInvitationByIdQuery: Send + Sync + 'static {
    async fn select_invitation_by_id(
        &self,
        params: &FindInvitationParams,
    ) -> Result<Invitation, BoxedError>;
}
