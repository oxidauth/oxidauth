use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::authorities::create_authority::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct CreateAuthorityReq {
    pub authority: CreateAuthority,
}

#[derive(Debug, Serialize)]
pub struct CreateAuthorityRes {
    pub authority: Authority,
}

#[tracing::instrument(name = "create_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(mut params): Json<CreateAuthorityReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<CreateAuthorityService>();

    info!("provided CreateAuthorityService");

    let result = service
        .call(&mut params.authority)
        .await;

    match result {
        Ok(authority) => {
            info!(
                message = "successfully created authority",
                authority = ?authority,
            );

            Response::success().payload(CreateAuthorityRes { authority })
        },
        Err(err) => {
            info!(
                message = "failed to create authority",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
