use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use oxidauth_kernel::authorities::AuthorityStrategy; // for oxidauth rs
use oxidauth_kernel::{auth::register::*, error::IntoOxidAuthError};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type RegisterReq = RegisterParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}

#[tracing::instrument(name = "register_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<RegisterReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<RegisterService>();

    info!("provided RegisterService");

    let result = service.call(&params).await;

    match result {
        Ok(response) => {
            info!(
                message = "successfully registered",
                response = ?response,
            );

            Response::success().payload(RegisterRes {
                jwt: response.jwt,
                refresh_token: response.refresh_token,
            })
        },
        Err(err) => {
            info!(
                message = "failed to register",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
