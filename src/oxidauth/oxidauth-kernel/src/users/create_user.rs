use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::{User, UserKind, UserStatus};

pub type CreateUserService = Arc<
    dyn for<'a> Service<&'a CreateUser, Response = User, Error = BoxedError>,
>;

pub trait CreateUserTrait:
    for<'a> Service<&'a CreateUser, Response = User, Error = BoxedError>
{
}

impl<T> CreateUserTrait for T where
    T: for<'a> Service<&'a CreateUser, Response = User, Error = BoxedError>
{
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub id: Option<Uuid>,
    pub kind: Option<UserKind>,
    pub status: Option<UserStatus>,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Option<Value>,
}
