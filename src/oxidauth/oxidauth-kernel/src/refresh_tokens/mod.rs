use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod create_refresh_token;
pub mod delete_refresh_token_by_id;
pub mod delete_refresh_token_by_user_id;
pub mod exchange_refresh_token;
pub mod find_refresh_token_by_id;

pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub authority_id: Uuid,
    /// `false` while the authority's second factor is still owed: the token was
    /// minted by a TOTP-enabled authority that never saw a validated code, so the
    /// exchange path may sign only the TOTP-validate entitlement. `true` is the
    /// ordinary case — every token from a TOTP-disabled authority, and every token
    /// outliving a validated one.
    pub totp_validated: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
