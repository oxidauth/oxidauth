use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::dev_prelude::Service;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordParams {
    pub code: String,
    pub password: String,
    pub password_conf: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordInfo {
    pub code: String,
    pub password: String,
    pub password_conf: String,
}

pub type ResetPasswordService =
    Arc<dyn for<'a> Service<&'a ResetPasswordParams, Response = String, Error = String>>;
