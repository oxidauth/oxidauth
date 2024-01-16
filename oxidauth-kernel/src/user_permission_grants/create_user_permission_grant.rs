use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::UserPermission;

pub type CreateUserPermissionGrantService = Arc<
    dyn for<'a> Service<
        &'a CreateUserPermission,
        Response = UserPermission,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct CreateUserPermission {
    pub user_id: Uuid,
    pub permission: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}
