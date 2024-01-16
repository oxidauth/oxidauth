use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::RoleRoleGrant;

pub type DeleteRoleRoleGrantService = Arc<
    dyn for<'a> Service<
        &'a DeleteRoleRoleGrant,
        Response = RoleRoleGrant,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteRoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}
