use std::sync::Arc;

use serde::{Serialize, Deserialize};
use serde_json::Value;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::{Authority, AuthorityStatus, AuthorityStrategy, AuthoritySettings};

pub type CreateAuthorityService = Arc<
    dyn for<'a> Service<
        &'a mut CreateAuthority,
        Response = Authority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthority {
    pub name: String,
    pub client_key: Option<Uuid>,
    pub status: Option<AuthorityStatus>,
    pub strategy: AuthorityStrategy,
    pub settings: AuthoritySettings,
    pub params: Value,
}
