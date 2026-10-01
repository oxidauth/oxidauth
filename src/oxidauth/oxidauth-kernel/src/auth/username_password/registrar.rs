use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    JsonValue,
    Password,
    dev_prelude::*,
    users::{UserKind, UserStatus, create_user::CreateUser},
};

/// Registration payload for the username/password authority strategy. Lives
/// in the kernel because it is a wire DTO: the SDK client (`accept_invitation`)
/// serializes it, the services registrar deserializes it — the wasm client
/// graph must not depend on the services crate (which carries the jwt crypto
/// and the use-case stack).
#[derive(Clone, Deserialize)]
pub struct UsernamePasswordRegisterParams {
    pub username: String,
    pub password: Password,
    pub password_confirmation: Password,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub kind: Option<UserKind>,
}

impl TryFrom<JsonValue> for UsernamePasswordRegisterParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;
        Ok(s)
    }
}

impl From<UsernamePasswordRegisterParams> for CreateUser {
    fn from(params: UsernamePasswordRegisterParams) -> Self {
        let UsernamePasswordRegisterParams {
            username,
            email,
            first_name,
            last_name,
            kind,
            ..
        } = params.clone();
        let user_id = Uuid::new_v4();
        let kind = Some(kind.unwrap_or_default());

        Self {
            id: Some(user_id),
            username,
            email,
            first_name,
            last_name,
            status: Some(UserStatus::default()),
            kind,
            profile: Some(Value::default()),
        }
    }
}
