use serde::{Serialize, Deserialize};
use std::sync::Arc;

pub use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type CreateRoleService = Arc<
    dyn for<'a> Service<
        &'a CreateRole,
        Response = Role,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRole {
    pub name: String,
}
