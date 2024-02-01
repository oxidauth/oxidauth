pub mod authenticator;
pub mod helpers;
pub mod registrar;
pub mod user_authority_from_request;
pub mod user_identifier_from_request;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use oxidauth_kernel::{error::BoxedError, JsonValue};

#[derive(Debug)]
pub struct UsernamePassword {
    authority_id: Uuid,
    params: AuthorityParams,
    password_pepper: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorityParams {
    password_salt: String,
}

impl AuthorityParams {
    pub fn new(password_salt: String) -> Self {
        Self { password_salt }
    }

    pub fn as_value(&self) -> Result<Value, BoxedError> {
        Ok(serde_json::to_value(self)?)
    }
}

impl TryFrom<JsonValue> for AuthorityParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserAuthorityParams {
    pub password_hash: String,
}

impl TryFrom<JsonValue> for UserAuthorityParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}
