use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthcheckRes {
    pub version: String,
    pub healthy: bool,
}
