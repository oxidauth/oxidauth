use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type FindRoleByNameService = Arc<
    dyn for<'a> Service<
        &'a FindRoleByName,
        Response = Role,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRoleByName {
    pub role: String,
}
