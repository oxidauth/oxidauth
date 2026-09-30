use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LivecheckRes {
    pub version: String,
    pub healthy: bool,
}
