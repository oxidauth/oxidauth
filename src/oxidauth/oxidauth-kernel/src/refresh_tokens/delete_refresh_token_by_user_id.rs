use serde::Deserialize;
use uuid::Uuid;

pub use super::RefreshToken;

#[derive(Debug, Deserialize)]
pub struct DeleteRefreshTokenByUserId {
    pub user_id: Uuid,
}
