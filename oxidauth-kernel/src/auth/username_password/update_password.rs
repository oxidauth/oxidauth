use crate::error::BoxedError;
pub use crate::service::Service;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

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

pub type UpdatePasswordService = Arc<
    dyn for<'a> Service<
            &'a UpdatePasswordParams,
            Response = UpdatePasswordResponse,
            Error = BoxedError,
        >,
>;
