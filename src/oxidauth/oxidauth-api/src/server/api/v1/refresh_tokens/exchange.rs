use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{Response, refresh_tokens::exchange::ExchangeRefreshTokenReq};
use oxidauth_kernel::{error::IntoOxidAuthError, refresh_tokens::exchange_refresh_token::*};
use tracing::info;

use crate::provider::Provider;

#[tracing::instrument(name = "exchange_refresh_token_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<ExchangeRefreshTokenReq>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<ExchangeRefreshTokenService>();

    info!("provided ExchangeRefreshTokenService");

    let result = service
        .exchange_refresh_token(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
