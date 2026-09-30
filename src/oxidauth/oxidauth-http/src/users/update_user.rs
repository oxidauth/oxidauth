use oxidauth_kernel::users::update_user::{User, UserStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UpdateUserPathReq {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserBodyReq {
    pub user: UpdateUserUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<UserStatus>,
    pub profile: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRes {
    pub user: User,
}
