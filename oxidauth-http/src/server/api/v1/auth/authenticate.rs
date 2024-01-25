use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::auth::authenticate::AuthenticateService;
use oxidauth_kernel::auth::authenticate::*;
use oxidauth_kernel::authorities::AuthorityStrategy;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct AuthenticateReq {
    pub strategy: AuthorityStrategy,
    pub params: Value,
}

#[allow(clippy::from_over_into)]
impl Into<AuthenticateParams> for AuthenticateReq {
    fn into(self) -> AuthenticateParams {
        AuthenticateParams {
            strategy: self.strategy,
            params: self.params,
        }
    }
}

#[derive(Debug, Serialize)]
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

    let result = service
        .call(&params.into())
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

            Response::fail().error(err.into_error())
        },
    }
}
