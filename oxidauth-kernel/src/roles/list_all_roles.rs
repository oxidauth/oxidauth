use serde::Deserialize;
use std::sync::Arc;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type ListAllRolesService = Arc<
    dyn for<'a> Service<
        &'a ListAllRoles,
        Response = Vec<Role>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct ListAllRoles {}

