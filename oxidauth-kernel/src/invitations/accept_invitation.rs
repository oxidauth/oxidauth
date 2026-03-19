use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    auth::register::RegisterParams,
    error::BoxedError,
    users::{update_user::UpdateUser, User, UserStatus},
};

#[async_trait]
pub trait AcceptInvitationTrait: Send + Sync + 'static {
    async fn accept_invitation(
        &self,
        params: &AcceptInvitationParams,
    ) -> Result<User, BoxedError>;
}

pub type AcceptInvitationService = Arc<dyn AcceptInvitationTrait>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptInvitationParams {
    pub invitation_id: Uuid,
    pub user: AcceptInvitationUserParams,
    pub user_authority: RegisterParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptInvitationUserParams {
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<UserStatus>,
    pub profile: Option<Value>,
}

impl
    From<(
        Uuid,
        &AcceptInvitationUserParams,
    )> for UpdateUser
{
    fn from(
        (user_id, value): (
            Uuid,
            &AcceptInvitationUserParams,
        ),
    ) -> Self {
        let value = value.clone();

        Self {
            id: user_id,
            username: Some(value.username),
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            status: value.status,
            profile: value.profile,
        }
    }
}
