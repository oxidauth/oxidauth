use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{provider::Provider, response::Response};
use oxidauth_kernel::auth::authenticate::{
    AuthenticateParams, AuthenticateService,
};
use oxidauth_kernel::error::IntoOxidAuthError;

pub type AuthenticateReq = AuthenticateParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}

#[tracing::instrument(name = "authenticate_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<AuthenticateReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<AuthenticateService>();

    info!("provided AuthenticateService");

    let result = service.call(&params).await;

    match result {
        Ok(response) => {
            info!(
                message = "successfully authenticated",
                response = ?response,
            );

            Response::success().payload(AuthenticateRes {
                jwt: response.jwt,
                refresh_token: response.refresh_token,
            })
        },
        Err(err) => {
            info!(
                message = "failed to authenticate",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
