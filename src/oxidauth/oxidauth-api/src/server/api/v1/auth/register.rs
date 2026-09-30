use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{
    Response,
    auth::register::{RegisterReq, RegisterRes},
};
use oxidauth_kernel::{auth::register::*, error::IntoOxidAuthError};
use tracing::info;

use crate::provider::Provider;

#[tracing::instrument(name = "register_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<RegisterReq>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<RegisterService>();

    info!("provided RegisterService");

    let result = service
        .register(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
