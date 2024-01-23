use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::RefreshToken;

pub type DeleteRefreshTokenService = Arc<
    dyn for<'a> Service<
        &'a DeleteRefreshToken,
        Response = RefreshToken,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteRefreshToken {
    pub refresh_token_id: Uuid,
}
