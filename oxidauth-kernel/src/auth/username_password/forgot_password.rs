use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::dev_prelude::Service;

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordParams {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordResponse {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordInfo {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordUseCaseRes {
    pub code: String,
}

pub type ForgotPasswordService = Arc<
    dyn for<'a> Service<&'a ForgotPasswordInfo, Response = ForgotPasswordResponse, Error = String>,
>;
