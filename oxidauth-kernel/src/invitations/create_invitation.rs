use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dev_prelude::{BoxedError, Service};

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
    pub user_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
}
