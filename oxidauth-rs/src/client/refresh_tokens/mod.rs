mod exchange_refresh_token;

#[cfg(feature = "mock")]
use super::mock::ClientMock;
use super::{
    Client,
    Resource,
    fmt,
    handle_response,
};
use crate::refresh_tokens::exchange_refresh_token::ExchangeRefreshTokenTrait;

pub trait RefreshTokensTrait: ExchangeRefreshTokenTrait {}

impl RefreshTokensTrait for Client {
}

#[cfg(feature = "mock")]
impl RefreshTokensTrait for ClientMock {
}
