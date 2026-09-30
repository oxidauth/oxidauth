use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
