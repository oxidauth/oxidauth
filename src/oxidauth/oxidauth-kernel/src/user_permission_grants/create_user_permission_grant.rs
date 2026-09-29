use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserPermission {
    pub user_id: Uuid,
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}
