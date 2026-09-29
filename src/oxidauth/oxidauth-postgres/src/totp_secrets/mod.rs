pub mod insert_totp_secret;
pub mod insert_totp_secrets;
pub mod select_totp_secret_by_user_id;
pub mod select_where_no_totp_secret_by_authority_id;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct PgTotpSecret {
    pub id: Uuid,
    pub user_id: Uuid,
    pub totp_secret: String,
    pub created_at: DateTime<Utc>,
}
