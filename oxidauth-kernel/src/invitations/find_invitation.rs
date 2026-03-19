use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;

use super::Invitation;

#[async_trait]
pub trait FindInvitationTrait: Send + Sync + 'static {
    async fn find_invitation(
        &self,
        params: &FindInvitationParams,
    ) -> Result<Invitation, BoxedError>;
}

pub type FindInvitationService = Arc<dyn FindInvitationTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindInvitationParams {
    pub invitation_id: Uuid,
}
