use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod generate;
pub mod validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct TOTPValidationRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}
