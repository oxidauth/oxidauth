use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;

use super::Invitation;

#[async_trait]
pub trait DeleteInvitationTrait: Send + Sync + 'static {
    async fn delete_invitation(
        &self,
        params: &DeleteInvitationParams,
    ) -> Result<Invitation, BoxedError>;
}

pub type DeleteInvitationService = Arc<dyn DeleteInvitationTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteInvitationParams {
    pub id: Uuid,
}
