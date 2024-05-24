use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod generate;
pub mod validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct TOTPCode {
    pub code: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TOTPGenerationRes {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TOTPValidationRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}
