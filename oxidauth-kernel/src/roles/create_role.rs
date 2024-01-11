use serde::Deserialize;
use std::sync::Arc;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type CreateRoleService = Arc<
    dyn for<'a> Service<
        &'a CreateRole,
        Response = Role,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct CreateRole {
    pub name: String,
}
