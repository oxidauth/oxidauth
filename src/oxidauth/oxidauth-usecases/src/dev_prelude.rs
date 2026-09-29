use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

pub use async_trait::async_trait;
pub use serde_json::Value;
pub use uuid::Uuid;

pub fn epoch() -> Result<i64, SystemTimeError> {
    let now = SystemTime::now();

    epoch_from_time(now)
}

pub fn epoch_from_time(t: SystemTime) -> Result<i64, SystemTimeError> {
    let epoch = t
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64;

    Ok(epoch)
}
