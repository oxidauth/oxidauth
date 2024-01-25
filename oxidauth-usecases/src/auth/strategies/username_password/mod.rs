pub mod authenticator;
pub mod helpers;
pub mod registrar;
pub mod user_authority_from_request;
pub mod user_identifier_from_request;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use oxidauth_kernel::error::BoxedError;

#[derive(Debug)]
pub struct UsernamePassword {
    authority_id: Uuid,
    params: AuthorityParams,
    password_pepper: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthorityParams {
    password_salt: String,
}

impl TryFrom<serde_json::Value> for AuthorityParams {
    type Error = BoxedError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value)?;

        Ok(s)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserAuthorityParams {
    pub password_hash: String,
}

impl TryFrom<serde_json::Value> for UserAuthorityParams {
    type Error = BoxedError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value)?;

        Ok(s)
    }
}
