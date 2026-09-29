pub mod fetch_setting;
pub mod save_setting;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
