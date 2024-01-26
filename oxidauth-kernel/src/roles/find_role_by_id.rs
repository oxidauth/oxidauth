use std::sync::Arc;

use serde::{Serialize, Deserialize};
use uuid::Uuid;

pub use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type FindRoleByIdService = Arc<
    dyn for<'a> Service<
        &'a FindRoleById,
        Response = Role,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRoleById {
    pub role_id: Uuid,
}
