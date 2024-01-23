use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod create_refresh_token;
pub mod exchange_refresh_token;
pub mod find_refresh_token_by_id;

pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
