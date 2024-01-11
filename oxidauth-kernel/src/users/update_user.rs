use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::{User, UserStatus};

pub type UpdateUserService = Arc<
    dyn for<'a> Service<&'a UpdateUser, Response = User, Error = BoxedError>,
>;

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub id: Option<Uuid>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<UserStatus>,
    pub profile: Option<Value>,
}
