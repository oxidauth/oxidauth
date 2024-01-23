use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::{error::BoxedError, auth::authenticate::AuthenticateResponse};
pub use crate::service::Service;

pub use super::RefreshToken;

pub type ExchangeRefreshTokenService = Arc<
    dyn for<'a> Service<
        &'a ExchangeRefreshToken,
        Response = AuthenticateResponse,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct ExchangeRefreshToken {
    pub refresh_token_id: Uuid,
}
