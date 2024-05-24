use std::fmt;

use serde::{Deserialize, Serialize};

pub mod create_totp_secret;
pub mod select_totp_secret_by_user_id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TOTPSecret {
    pub secret: String,
}

impl fmt::Display for TOTPSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Could not find TOTP secret by user id",
        )
    }
}
