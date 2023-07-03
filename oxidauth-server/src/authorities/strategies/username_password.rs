use std::error::Error as StdError;

use argon2::{
    password_hash::{Error as PasswordHashError, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use async_trait::async_trait;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    authorities::{Authenticator, Authority, Registrar},
    axum::server::routes::api::v1::{
        authorities::all::AuthorityRow,
        users::{
            authorities::all::UserAuthority, authorities::create::UserAuthorityCreate,
            create::UserCreateRow,
        },
    },
};

pub type Error = Box<dyn StdError + Send + Sync + 'static>;

pub struct UsernamePassword {
    authority_id: Uuid,
    params: AuthorityParams,
    password_pepper: String,
}

pub async fn new(authority: &AuthorityRow) -> Result<Box<dyn Authority>, Error> {
    let params: AuthorityParams = authority.params.clone().try_into()?;
    let authority_id = authority.id;
    let password_pepper = std::env::var("OXIDAUTH_USERNAME_PASSWORD_PEPPER")?;

    Ok(Box::new(UsernamePassword {
        authority_id,
        params,
        password_pepper,
    }))
}

#[async_trait]
impl Registrar for UsernamePassword {
    async fn user_authority_from_request(
        &self,
        authority_params: Value,
    ) -> Result<UserAuthorityCreate, Error> {
        let authority_params: AuthenticateParams = authority_params.clone().try_into()?;

        let password = format!(
            "{}:{}:{}",
            authority_params.password, self.params.password_salt, self.password_pepper,
        );

        let password_hash = hash_password(password).map_err(|err| err.to_string())?;
        let params = UserAuthorityParams { password_hash };

        let params = serde_json::to_value(params)?;

        let user_authority = UserAuthorityCreate {
            authority_id: self.authority_id,
            user_identifier: authority_params.username.clone(),
            params,
        };

        Ok(user_authority)
    }

    async fn register(
        &self,
        register_params: Value,
    ) -> Result<(UserCreateRow, UserAuthorityCreate), Error> {
        let register_params: RegisterParams = register_params.clone().try_into()?;

        if register_params.password != register_params.password_confirmation {
            return Err("password and password confirmation do not match".into());
        }

        let user: UserCreateRow = register_params.clone().into();

        let password = format!(
            "{}:{}:{}",
            register_params.password, self.params.password_salt, self.password_pepper,
        );

        let password_hash = hash_password(password).map_err(|err| err.to_string())?;
        let params = UserAuthorityParams { password_hash };

        let params = serde_json::to_value(params)?;

        let user_authority = UserAuthorityCreate {
            authority_id: self.authority_id,
            user_identifier: user.username.clone(),
            params,
        };

        Ok((user, user_authority))
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RegisterParams {
    pub username: String,
    pub password: String,
    pub password_confirmation: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl TryFrom<Value> for RegisterParams {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value)?;

        Ok(s)
    }
}

impl From<RegisterParams> for UserCreateRow {
    fn from(params: RegisterParams) -> Self {
        let RegisterParams {
            username,
            email,
            first_name,
            last_name,
            ..
        } = params.clone();
        let status = Some("active".to_string());
        let kind = Some("human".to_string());

        Self {
            username,
            email,
            first_name,
            last_name,
            status,
            kind,
            profile: Some(Value::default()),
        }
    }
}

#[async_trait]
impl Authenticator for UsernamePassword {
    async fn user_identifier_from_request(&self, params: Value) -> Result<String, Error> {
        let AuthenticateParams { username, .. } = serde_json::from_value(params)?;

        Ok(username)
    }

    async fn authenticate(
        &self,
        authenticate_params: Value,
        user_authority: &UserAuthority,
    ) -> Result<(), Error> {
        let authenticate_params: AuthenticateParams = authenticate_params.clone().try_into()?;

        let password = format!(
            "{}:{}:{}",
            authenticate_params.password, self.params.password_salt, self.password_pepper,
        );

        let user_authority_params: UserAuthorityParams =
            user_authority.params.clone().try_into()?;

        verify_password(password, user_authority_params.password_hash)
            .map_err(|err| err.to_string())?;

        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AuthenticateParams {
    pub username: String,
    pub password: String,
}

impl TryFrom<Value> for AuthenticateParams {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value)?;

        Ok(s)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UserAuthorityParams {
    pub password_hash: String,
}

impl TryFrom<Value> for UserAuthorityParams {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value)?;

        Ok(s)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AuthorityParams {
    password_salt: String,
}

impl TryFrom<Value> for AuthorityParams {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value)?;

        Ok(s)
    }
}

pub fn hash_password(password: String) -> Result<String, PasswordHashError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(&password.into_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: String, password_hash: String) -> Result<bool, PasswordHashError> {
    let password_hash = PasswordHash::new(&password_hash)?;

    Argon2::default().verify_password(&password.into_bytes(), &password_hash)?;

    Ok(true)
}
