use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::{Authority, AuthorityStatus, AuthorityStrategy, AuthoritySettings};

pub type UpdateAuthorityService = Arc<
    dyn for<'a> Service<
        &'a mut UpdateAuthority,
        Response = Authority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct UpdateAuthority {
    pub id: Option<Uuid>,
    pub name: String,
    pub client_key: Option<Uuid>,
    pub status: Option<AuthorityStatus>,
    pub strategy: AuthorityStrategy,
    pub settings: AuthoritySettings,
    pub params: Value,
}
