mod exchange_refresh_token;

#[cfg(feature = "mock")]
use super::mock::ClientMock;

use super::{Client, Resource, fmt, handle_response};

pub use crate::refresh_tokens::exchange_refresh_token::ExchangeRefreshTokenTrait;
pub use oxidauth_http::server::api::v1::refresh_tokens::exchange::{
    ExchangeRefreshTokenReq, ExchangeRefreshTokenRes,
};

pub trait RefreshTokensTrait: ExchangeRefreshTokenTrait {}

impl RefreshTokensTrait for Client {}

#[cfg(feature = "mock")]
impl RefreshTokensTrait for ClientMock {}
