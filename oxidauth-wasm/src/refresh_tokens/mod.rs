mod exchange_refresh_token;

#[cfg(feature = "mock")]
use super::mock::ClientMock;

use super::{Client, Resource, fmt, handle_response};

use crate::BoxedError;
pub use crate::refresh_tokens::exchange_refresh_token::ExchangeRefreshTokenTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeRefreshTokenReq {
    pub refresh_token: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeRefreshTokenRes {
    pub jwt: String,
    pub refresh_token: Uuid,
    pub user_id: Uuid,
}

pub trait RefreshTokensTrait: ExchangeRefreshTokenTrait {}

impl RefreshTokensTrait for Client {}

#[cfg(feature = "mock")]
impl RefreshTokensTrait for ClientMock {}
