use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

use super::AuthoritySettings;
pub use super::{Authority, AuthorityStatus, AuthorityStrategy};

pub type CreateAuthorityService = Arc<
    dyn for<'a> Service<
        &'a CreateAuthority,
        Response = Authority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct CreateAuthority {
    pub name: String,
    pub client_key: Uuid,
    pub status: AuthorityStatus,
    pub strategy: AuthorityStrategy,
    pub settings: AuthoritySettings,
    pub params: Value,
}
