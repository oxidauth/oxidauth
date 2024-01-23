use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

pub use crate::service::Service;
use crate::{auth::authenticate::AuthenticateResponse, error::BoxedError};

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
    pub refresh_token: Uuid,
}
