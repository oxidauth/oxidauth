use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    auth::register::RegisterParams,
    dev_prelude::BoxedError,
    users::{User, update_user::UpdateUser},
};

#[async_trait]
pub trait AcceptInvitationServiceTrait: Send + Sync + 'static {
    async fn accept_invitation(&self, params: &AcceptInvitationParams) -> Result<User, BoxedError>;
}

pub type AcceptInvitationService = Arc<dyn AcceptInvitationServiceTrait>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptInvitationParams {
    pub invitation_id: Uuid,
    pub user: AcceptInvitationUserParams,
    pub user_authority: RegisterParams,
}

/// OXA-000009: there is deliberately no `status` field — accepting an
/// invitation must never let the invitee write their own account status. The
/// stored value is backfilled by `UpdateUserUseCase`, and status transitions
/// belong to the admin `update_user` route only. Serde is permissive (no
/// `deny_unknown_fields` anywhere in the workspace), so existing clients that
/// still send `"status"` keep deserialising — the field just stops being
/// honoured (behaviour change, see changelog).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptInvitationUserParams {
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Option<Value>,
}

impl From<(Uuid, &AcceptInvitationUserParams)> for UpdateUser {
    fn from((user_id, value): (Uuid, &AcceptInvitationUserParams)) -> Self {
        let value = value.clone();

        Self {
            id: user_id,
            username: Some(value.username),
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            // OXA-000009: `None` lets `UpdateUserUseCase` backfill the stored
            // status — the invitation flow never changes it.
            status: None,
            profile: value.profile,
        }
    }
}
