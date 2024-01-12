use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::{error::BoxedError, roles::Role};
pub use crate::service::Service;

pub use super::RoleRoleGrant;

pub type CreateRoleRoleGrantService = Arc<
    dyn for<'a> Service<
        &'a CreateRoleRoleGrant,
        Response = CreateRoleRoleGrantResponse,
        Error = BoxedError,
    >,
>;

#[derive(Debug)]
pub struct CreateRoleRoleGrantResponse {
    pub child: Role,
    pub grant: RoleRoleGrant
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}

