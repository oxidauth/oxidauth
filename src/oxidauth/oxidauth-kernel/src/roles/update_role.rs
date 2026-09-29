use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Role;

pub type UpdateRoleService = Arc<
    dyn for<'a> Service<&'a UpdateRole, Response = Role, Error = BoxedError>,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRole {
    pub role_id: Option<Uuid>,
    pub name: String,
}
