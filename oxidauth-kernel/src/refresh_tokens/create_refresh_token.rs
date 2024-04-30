use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::RefreshToken;

pub type CreateRefreshTokenService = Arc<
    dyn for<'a> Service<
        &'a CreateRefreshToken,
        Response = RefreshToken,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct CreateRefreshToken {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub expires_at: DateTime<Utc>,
}
