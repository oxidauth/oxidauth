use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    dev_prelude::{BoxedError, Service},
    users::{update_user::UpdateUser, User, UserStatus},
};

pub type AcceptInvitationService = Arc<
    dyn for<'a> Service<
        &'a AcceptInvitationParams,
        Response = User,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptInvitationParams {
    pub invitation_id: Uuid,
    pub user: AcceptInvitationUserParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptInvitationUserParams {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<UserStatus>,
    pub profile: Option<Value>,
}

impl From<&AcceptInvitationUserParams> for UpdateUser {
    fn from(value: &AcceptInvitationUserParams) -> Self {
        let value = value.clone();

        Self {
            id: value.id,
            username: Some(value.username),
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            status: value.status,
            profile: value.profile,
        }
    }
}
