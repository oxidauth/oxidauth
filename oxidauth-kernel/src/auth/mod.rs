pub mod authenticate;
pub mod oauth2;
pub mod register;
pub mod tree;

use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    JsonValue, authorities::UserAuthority, dev_prelude::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority,
    users::create_user::CreateUser,
};

#[async_trait]
pub trait Registrar: UserAuthorityFromRequest + Send + Sync + 'static {
    async fn register(
        &self,
        params: JsonValue,
    ) -> Result<
        (
            CreateUser,
            CreateUserAuthority,
        ),
        BoxedError,
    >;
}

#[async_trait]
pub trait UserAuthorityFromRequest: Send + Sync + 'static {
    async fn user_authority_from_request(
        &self,
        params: JsonValue,
    ) -> Result<CreateUserAuthority, BoxedError>;
}

#[async_trait]
pub trait Authenticator:
    UserIdentifierFromRequest + Send + Sync + 'static
{
    async fn authenticate(
        &self,
        params: JsonValue,
        user_authority: &UserAuthority,
    ) -> Result<(), BoxedError>;
}

#[async_trait]
pub trait UserIdentifierFromRequest: Send + Sync + 'static {
    async fn user_identifier_from_request(
        &self,
        request: &JsonValue,
    ) -> Result<String, BoxedError>;
}

#[derive(Debug, Deserialize)]
pub struct FindRedirectUrlByAuthorityClientKey {
    pub authority_client_key: Uuid,
}

#[derive(Debug)]
pub struct FindRedirectUrlByAuthorityClientKeyError {}
