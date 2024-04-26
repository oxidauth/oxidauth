use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Authority;

pub type FindAuthorityByClientKeyService = Arc<
    dyn for<'a> Service<
        &'a FindAuthorityByClientKey,
        Response = Authority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAuthorityByClientKey {
    pub client_key: Uuid,
}
