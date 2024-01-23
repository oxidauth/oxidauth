use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::RefreshToken;

pub type FindRefreshTokenByIdService = Arc<
    dyn for<'a> Service<
        &'a FindRefreshTokenById,
        Response = RefreshToken,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct FindRefreshTokenById {
    pub refresh_token_id: Uuid,
}
