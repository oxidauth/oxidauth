use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::refresh_tokens::exchange_refresh_token::*;
use serde::Serialize;
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

pub type ExchangeRefreshTokenReq = ExchangeRefreshToken;

#[derive(Debug, Serialize)]
pub struct ExchangeRefreshTokenRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}

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

            Response::success().payload(ExchangeRefreshTokenRes {
                jwt: result.jwt,
                refresh_token: result.refresh_token,
            })
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