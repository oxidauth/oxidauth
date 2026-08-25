pub mod validate;

use crate::BoxedError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::response::Response;

use super::{Client, Resource, fmt, handle_response};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTOTPReq {
    pub code: String,
    pub client_key: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTOTPRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}
