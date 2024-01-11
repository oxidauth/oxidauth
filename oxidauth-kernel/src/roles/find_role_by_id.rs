use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type FindRoleByIdService = Arc<
    dyn for<'a> Service<
        &'a FindRoleById,
        Response = Role,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct FindRoleById {
    pub role_id: Uuid,
}
