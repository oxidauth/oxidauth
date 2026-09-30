use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Invitation;
use crate::{
    dev_prelude::BoxedError,
    users::{User, create_user::CreateUser},
};

#[async_trait]
pub trait CreateInvitationServiceTrait: Send + Sync + 'static {
    async fn create_invitation(
        &self,
        params: &CreateInvitationParams,
    ) -> Result<CreateInvitationResponse, BoxedError>;
}

pub type CreateInvitationService = Arc<dyn CreateInvitationServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvitationResponse {
    pub invitation: Invitation,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvitationParams {
    pub id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub user: CreateUser,
}

impl From<CreateUser> for CreateInvitationParams {
    fn from(user: CreateUser) -> Self {
        Self {
            id: None,
            expires_at: None,
            user,
        }
    }
}
