use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::RefreshToken;

pub type DeleteRefreshTokenByIdService = Arc<
    dyn for<'a> Service<&'a DeleteRefreshTokenById, Response = RefreshToken, Error = BoxedError>,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteRefreshTokenById {
    pub refresh_token_id: Uuid,
}
