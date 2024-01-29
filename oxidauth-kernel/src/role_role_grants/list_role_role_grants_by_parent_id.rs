use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::RoleRoleGrant;
pub use super::RoleRoleGrantDetail;

pub type ListRoleRoleGrantsByParentIdService = Arc<
    dyn for<'a> Service<
        &'a ListRoleRoleGrantsByParentId,
        Response = Vec<RoleRoleGrantDetail>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRoleRoleGrantsByParentId {
    pub parent_id: Uuid,
}
