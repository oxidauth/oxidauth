use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Permission;

pub type FindPermissionByPartsService = Arc<
    dyn for<'a> Service<
        &'a FindPermissionByParts,
        Response = Permission,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPermissionByParts {
    pub permission: String,
}
