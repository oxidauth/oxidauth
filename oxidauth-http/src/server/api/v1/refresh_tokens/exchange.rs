use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::auth::authenticate::AuthenticateResponse;
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::refresh_tokens::exchange_refresh_token::*;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type ExchangeRefreshTokenReq = ExchangeRefreshToken;

type ExchangeRefreshTokenRes = AuthenticateResponse;

#[tracing::instrument(name = "exchange_refresh_token_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<ExchangeRefreshTokenReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ExchangeRefreshTokenService>();

    info!("provided ExchangeRefreshTokenService");

    let result = service.call(&params).await;

    match result {
        Ok(result) => {
            info!(
                message = "successfully exchanged refresh token",
                result = ?result,
            );

            Response::success().payload(result)
        },
        Err(err) => {
            info!(
                message = "failed to exchange refresh token",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
