use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::UserRole;

pub type CreateUserRoleGrantService = Arc<
    dyn for<'a> Service<
        &'a CreateUserRoleGrant,
        Response = UserRole,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct CreateUserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
}
