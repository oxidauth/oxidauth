use async_trait::async_trait;
use chrono::{DateTime, Utc};
use oxidauth_kernel::{error::BoxedError, invitations::Invitation};
use uuid::Uuid;

#[async_trait]
pub trait InsertInvitationQuery: Send + Sync + 'static {
    async fn insert_invitation(
        &self,
        params: &InsertInvitationParams,
    ) -> Result<Invitation, BoxedError>;
}
#[derive(Debug)]
pub struct InsertInvitationParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}
