pub mod insert_totp_secret;
pub mod select_totp_secret_by_user_id;

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct TOTPSecretRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub totp_secret: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PgTotpSecret {
    pub id: Uuid,
    pub user_id: Uuid,
    pub totp_secret: String,
    pub created_at: DateTime<Utc>,
}

impl From<PgTotpSecret> for TOTPSecretRow {
    fn from(value: PgTotpSecret) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            totp_secret: value.totp_secret,
            created_at: value.created_at,
        }
    }
}
