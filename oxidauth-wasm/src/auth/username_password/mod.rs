pub mod forgot_password;
pub mod update_password;

use super::{Client, fmt};

use crate::BoxedError;
use crate::response::Response;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordParams {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordResponse {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordParams {
    pub code: String,
    pub username: String,
    pub client_key: Uuid,
    pub password: String,
    pub password_conf: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordResponse {
    pub success: bool,
}
