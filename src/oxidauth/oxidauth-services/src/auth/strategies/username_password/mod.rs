pub mod authenticator;
pub mod forgot_password;
pub mod helpers;
pub mod registrar;
pub mod update_password;
pub mod user_authority_from_request;
pub mod user_identifier_from_request;

use oxidauth_kernel::{JsonValue, error::BoxedError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    pub fn as_json_value(&self) -> Result<JsonValue, BoxedError> {
        Ok(JsonValue::new(serde_json::to_value(self)?))
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

#[cfg(test)]
pub(crate) mod fixtures {
    use std::time::Duration;

    use chrono::Utc;
    use oxidauth_kernel::{
        JsonValue,
        authorities::{
            Authority,
            AuthoritySettings,
            AuthorityStatus,
            AuthorityStrategy,
            TotpSettings,
        },
        jwt::EntitlementsEncoding,
        user_authorities::UserAuthority,
    };
    use uuid::Uuid;

    use super::{AuthorityParams, UsernamePassword};
    use crate::auth::strategies::username_password::helpers::{hash_password, raw_password_hash};

    pub(crate) const AUTHORITY_ID: Uuid = uuid::uuid!("2a4d5f82-e0e6-4a3f-9c3e-5d6b7a8c9d01");
    pub(crate) const USER_ID: Uuid = uuid::uuid!("3b5e6a93-f1f7-4b4a-8d2f-6e7c8b9d0e02");
    pub(crate) const CLIENT_KEY: Uuid = uuid::uuid!("5d708cb5-b3b9-4d6c-8f40-7f8d9cae1f03");

    pub(crate) const PASSWORD_SALT: &str = "unit-test-password-salt";
    pub(crate) const PASSWORD_PEPPER: &str = "unit-test-password-pepper";

    pub(crate) fn username_password() -> UsernamePassword {
        username_password_with(PASSWORD_SALT, PASSWORD_PEPPER)
    }

    pub(crate) fn username_password_with(salt: &str, pepper: &str) -> UsernamePassword {
        UsernamePassword {
            authority_id: AUTHORITY_ID,
            params: AuthorityParams::new(salt.to_owned()),
            password_pepper: pepper.to_owned(),
        }
    }

    pub(crate) fn authority_params_json() -> JsonValue {
        AuthorityParams::new(PASSWORD_SALT.to_owned())
            .as_json_value()
            .expect("authority params serialize to json")
    }

    pub(crate) fn authority(params: JsonValue) -> Authority {
        Authority {
            id: AUTHORITY_ID,
            name: "username-password-authority".to_owned(),
            client_key: CLIENT_KEY,
            status: AuthorityStatus::Enabled,
            strategy: AuthorityStrategy::UsernamePassword,
            settings: AuthoritySettings {
                jwt_ttl: Duration::from_secs(900),
                jwt_nbf_offset: Default::default(),
                refresh_token_ttl: Duration::from_secs(86_400),
                totp: TotpSettings::Disabled,
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
            params,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub(crate) fn user_authority(params: JsonValue) -> UserAuthority {
        UserAuthority {
            user_id: USER_ID,
            authority_id: AUTHORITY_ID,
            user_identifier: "test-user".to_owned(),
            params,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Hashes `password` the way registration persists it, so authentication
    /// tests can seed a user authority from the same raw material.
    pub(crate) fn stored_password_hash(password: &str, salt: &str, pepper: &str) -> String {
        hash_password(raw_password_hash(password, salt, pepper))
            .expect("argon2 hashing should succeed")
    }
}
