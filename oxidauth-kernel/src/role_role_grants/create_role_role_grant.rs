use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::RoleRoleGrant;
pub use super::RoleRoleGrantDetail;

pub type CreateRoleRoleGrantService = Arc<
    dyn for<'a> Service<
        &'a CreateRoleRoleGrant,
        Response = RoleRoleGrantDetail,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}
