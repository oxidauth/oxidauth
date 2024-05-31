use serde::{Deserialize, Serialize};

pub mod create_totp_secret;
pub mod find_totp_secret_by_user_id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TOTPSecret {
    pub secret: String,
}
