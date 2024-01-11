use serde::Deserialize;
use std::sync::Arc;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Permission;

pub type DeletePermissionService = Arc<
    dyn for<'a> Service<
        &'a DeletePermission,
        Response = Permission,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeletePermission {
    pub permission: String,
}
