use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dev_prelude::{BoxedError, Service};

use super::Invitation;

pub type FindInvitationService = Arc<
    dyn for<'a> Service<
        &'a FindInvitationParams,
        Response = Invitation,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindInvitationParams {
    pub id: Uuid,
}
