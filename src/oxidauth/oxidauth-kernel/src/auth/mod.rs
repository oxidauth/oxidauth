pub mod authenticate;
pub mod authenticate_or_register;
pub mod oauth2;
pub mod register;
pub mod tree;
pub mod username_password;

use std::{error::Error, fmt};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    JsonValue,
    authorities::{Authority, AuthorityStrategy, UserAuthority},
    dev_prelude::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority,
    users::create_user::CreateUser,
};

/// Distinguishes caller-driven (HTTP/deserialized) authenticate & register
/// requests from trusted in-process hops.
///
/// The field carrying this entry is `serde(skip)`ped on
/// `crate::auth::authenticate::AuthenticateParams` and
/// `crate::auth::register::RegisterParams`, so
/// deserializing a wire body ALWAYS yields `External` — a request body can
/// never set `Internal` (OXA-000004 step 1). Only in-process code that builds
/// the params struct directly — e.g. `AuthenticateOrRegisterUseCase` entering
/// the generic pipeline after completing the IdP code exchange — may set
/// `Internal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthEntry {
    #[default]
    External,
    Internal,
}

/// Raised when an `AuthEntry::External` request targets an authority whose
/// strategy may not be driven through the generic authenticate/register
/// endpoints (OXA-000004 step 1: the oauth2 strategy verifies nothing in its
/// `Authenticator` hook — its real verification lives in the oauth2 callback
/// flow, so a generic-entry request against an oauth2 authority is a bypass
/// attempt).
#[derive(Debug)]
pub struct StrategyEntryNotAllowedError {
    pub client_key: Uuid,
    pub strategy: AuthorityStrategy,
}

impl StrategyEntryNotAllowedError {
    pub fn new(client_key: Uuid, strategy: AuthorityStrategy) -> Self {
        Self {
            client_key,
            strategy,
        }
    }
}

impl fmt::Display for StrategyEntryNotAllowedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.strategy {
            AuthorityStrategy::Oauth2 => {
                write!(
                    f,
                    "the {} strategy may not authenticate or register through the generic \
                 endpoint; oauth2 authorities authenticate via \
                 GET /v1/auth/oauth2/callback/{}",
                    self.strategy, self.client_key,
                )
            },
            strategy => {
                write!(
                    f,
                    "the {} strategy may not authenticate or register through the generic \
                 endpoint (client_key: {})",
                    strategy, self.client_key,
                )
            },
        }
    }
}

impl Error for StrategyEntryNotAllowedError {
}

#[async_trait]
pub trait Registrar: UserAuthorityFromRequest + Send + Sync + 'static {
    async fn register(
        &self,
        params: JsonValue,
    ) -> Result<(CreateUser, CreateUserAuthority), BoxedError>;
}

#[async_trait]
pub trait UserAuthorityFromRequest: Send + Sync + 'static {
    async fn user_authority_from_request(
        &self,
        params: JsonValue,
    ) -> Result<CreateUserAuthority, BoxedError>;
}

#[async_trait]
pub trait Authenticator: UserIdentifierFromRequest + Send + Sync + 'static {
    async fn authenticate(
        &self,
        params: JsonValue,
        authority: &Authority,
        user_authority: &UserAuthority,
    ) -> Result<(), BoxedError>;
}

#[async_trait]
pub trait UserIdentifierFromRequest: Send + Sync + 'static {
    async fn user_identifier_from_request(&self, request: &JsonValue)
    -> Result<String, BoxedError>;
}

#[derive(Debug, Deserialize)]
pub struct FindRedirectUrlByAuthorityClientKey {
    pub authority_client_key: Uuid,
}

#[derive(Debug)]
pub struct FindRedirectUrlByAuthorityClientKeyError {}
