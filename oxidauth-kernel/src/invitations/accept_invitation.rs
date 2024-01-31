use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    authorities::AuthorityStrategy,
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
    pub user_authority: AcceptInvitationUserAuthorityParams,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptInvitationUserAuthorityParams {
    pub strategy: AuthorityStrategy,
    pub params: Value,
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
