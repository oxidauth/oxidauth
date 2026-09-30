use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CanReq {
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanRes {}
