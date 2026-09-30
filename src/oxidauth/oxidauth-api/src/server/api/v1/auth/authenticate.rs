use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{
    Response,
    auth::authenticate::{AuthenticateReq, AuthenticateRes},
};
use oxidauth_kernel::{auth::authenticate::AuthenticateService, error::IntoOxidAuthError};
use tracing::info;

use crate::provider::Provider;

#[tracing::instrument(name = "authenticate_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<AuthenticateReq>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<AuthenticateService>();

    info!("provided AuthenticateService");

    let result = service
        .authenticate(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
