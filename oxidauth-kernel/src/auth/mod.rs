pub mod authenticate;
pub mod tree;

use async_trait::async_trait;
use serde_json::Value;

use crate::{authorities::UserAuthority, dev_prelude::BoxedError};

#[async_trait]
pub trait Registrar: UserIdentifierFromRequest + Send + Sync + 'static {
    async fn register(&self, params: Value);
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
