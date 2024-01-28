use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    dev_prelude::{BoxedError, Service},
    users::create_user::CreateUser,
};

use super::Invitation;

pub type CreateInvitationService = Arc<
    dyn for<'a> Service<
        &'a CreateInvitationParams,
        Response = Invitation,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvitationParams {
    pub id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub user: CreateUser,
}
