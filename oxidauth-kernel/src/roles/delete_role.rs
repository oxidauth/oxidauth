use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

pub use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type DeleteRoleService = Arc<
    dyn for<'a> Service<
        &'a DeleteRole,
        Response = Role,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRole {
    pub role_id: Uuid,
}
