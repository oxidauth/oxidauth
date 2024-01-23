pub mod authenticate;
pub mod tree;

use async_trait::async_trait;
use serde_json::Value;

use crate::{
    authorities::UserAuthority, dev_prelude::BoxedError,
    user_authorities::user_authority_create::UserAuthorityCreate,
};

#[async_trait]
pub trait Registrar: UserAuthorityFromRequest + Send + Sync + 'static {
    async fn register(&self, params: Value);
}

#[async_trait]
pub trait UserAuthorityFromRequest: Send + Sync + 'static {
    async fn user_authority_from_request(
        &self,
        params: Value,
    ) -> Result<UserAuthorityCreate, BoxedError>;
}

#[async_trait]
pub trait Authenticator:
    UserIdentifierFromRequest + Send + Sync + 'static
{
    async fn authenticate(
        &self,
        params: Value,
        user_authority: &UserAuthority,
    ) -> Result<(), BoxedError>;
}

#[async_trait]
pub trait UserIdentifierFromRequest: Send + Sync + 'static {
    async fn user_identifier_from_request(
        &self,
        request: &Value,
    ) -> Result<String, BoxedError>;
}
