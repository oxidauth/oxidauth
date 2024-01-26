use serde::{Serialize, Deserialize};
use std::sync::Arc;

pub use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type ListAllRolesService = Arc<
    dyn for<'a> Service<
        &'a ListAllRoles,
        Response = Vec<Role>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllRoles;
