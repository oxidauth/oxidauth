use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    dev_prelude::{BoxedError, Service},
    users::{create_user::CreateUser, User},
};

use super::Invitation;

pub type CreateInvitationService = Arc<
    dyn for<'a> Service<
        &'a CreateInvitationParams,
        Response = CreateInvitationResponse,
        Error = BoxedError,
    >,
>;

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
