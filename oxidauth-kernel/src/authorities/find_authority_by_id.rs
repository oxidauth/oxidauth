use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Authority;

pub type FindAuthorityByIdService = Arc<
    dyn for<'a> Service<
        &'a FindAuthorityById,
        Response = Authority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct FindAuthorityById {
    pub authority_id: Uuid,
}
