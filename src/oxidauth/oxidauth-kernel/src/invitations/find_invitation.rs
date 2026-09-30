use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Invitation;
use crate::dev_prelude::BoxedError;

#[async_trait]
pub trait FindInvitationServiceTrait: Send + Sync + 'static {
    async fn find_invitation(
        &self,
        params: &FindInvitationParams,
    ) -> Result<Invitation, BoxedError>;
}

pub type FindInvitationService = Arc<dyn FindInvitationServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindInvitationParams {
    pub invitation_id: Uuid,
}
