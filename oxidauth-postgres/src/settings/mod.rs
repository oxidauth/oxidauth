use chrono::{DateTime, Utc};
use oxidauth_kernel::settings::Setting;
use serde_json::Value;

pub mod upsert_setting;

#[derive(Debug, sqlx::FromRow)]
pub struct PgSetting {
    pub key: String,
    pub value: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgSetting> for Setting {
    fn from(value: PgSetting) -> Self {
        Self {
            key: value.key,
            value: value.value,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
