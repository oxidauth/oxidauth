use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Authority;

pub type DeleteAuthorityService = Arc<
    dyn for<'a> Service<
        &'a DeleteAuthority,
        Response = Authority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteAuthority {
    pub authority_id: Uuid,
}
