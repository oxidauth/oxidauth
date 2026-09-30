use oxidauth_kernel::{
    auth::authenticate::AuthenticateResponse,
    refresh_tokens::exchange_refresh_token::ExchangeRefreshToken,
};

pub type ExchangeRefreshTokenReq = ExchangeRefreshToken;

pub type ExchangeRefreshTokenRes = AuthenticateResponse;
