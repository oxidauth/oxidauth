use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Invitation;
use crate::dev_prelude::BoxedError;

#[async_trait]
pub trait DeleteInvitationServiceTrait: Send + Sync + 'static {
    async fn delete_invitation(
        &self,
        params: &DeleteInvitationParams,
    ) -> Result<Invitation, BoxedError>;
}

pub type DeleteInvitationService = Arc<dyn DeleteInvitationServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteInvitationParams {
    pub id: Uuid,
}
