use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::service::Service;
use crate::{error::BoxedError, JsonValue};

pub use super::{
    Authority, AuthoritySettings, AuthorityStatus, AuthorityStrategy,
};

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
    pub params: JsonValue,
}
