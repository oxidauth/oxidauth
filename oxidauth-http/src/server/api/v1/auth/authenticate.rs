use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::auth::authenticate::{
    AuthenticateParams, AuthenticateService,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{provider::Provider, response::Response};

pub type AuthenticateReq = AuthenticateParams;

#[derive(Debug, Deserialize, Serialize)]
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

    info!("provided RegisterService");

    let result = service.call(&params).await;

    match result {
        Ok(response) => {
            info!(message = "successfully authenticated",);

            Response::success().payload(AuthenticateRes {
                jwt: response.jwt,
                refresh_token: response.refresh_token,
            })
        },
        Err(err) => {
            info!(message = "failed to authenticate",);

            Response::fail().error(err.into_error())
        },
    }
}
