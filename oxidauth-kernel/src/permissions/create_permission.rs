use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Permission;

pub type CreatePermissionService = Arc<
    dyn for<'a> Service<
        &'a CreatePermission,
        Response = Permission,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePermission {
    pub permission: String,
}

