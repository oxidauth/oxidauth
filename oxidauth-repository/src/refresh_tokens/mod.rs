pub mod delete_refresh_token_by_expires_at;
pub mod delete_refresh_token;
pub mod insert_refresh_token;
pub mod select_refresh_token_by_id;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct RefreshTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
