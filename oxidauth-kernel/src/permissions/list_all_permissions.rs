use serde::Deserialize;
use std::sync::Arc;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Permission;

pub type ListAllPermissionsService = Arc<
    dyn for<'a> Service<
        &'a ListAllPermissions,
        Response = Vec<Permission>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct ListAllPermissions {}

