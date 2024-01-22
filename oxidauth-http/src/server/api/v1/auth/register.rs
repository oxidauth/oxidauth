use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::auth::register::RegisterService;
use oxidauth_kernel::auth::register::*;
use oxidauth_kernel::authorities::AuthorityStrategy;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct RegisterReq {
    pub strategy: AuthorityStrategy,
    pub params: Value,
}

#[allow(clippy::from_over_into)]
impl Into<RegisterParams> for RegisterReq {
    fn into(self) -> RegisterParams {
        RegisterParams {
            strategy: self.strategy,
            params: self.params,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RegisterRes {
    pub response: RegisterResponse,
}

#[tracing::instrument(name = "register_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<RegisterReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<RegisterService>();

    info!("provided RegisterService");

    let result = service
        .call(&params.into())
        .await;

    match result {
        Ok(response) => {
            info!(
                message = "successfully registered",
                response = ?response,
            );

            Response::success().payload(RegisterRes { response })
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
