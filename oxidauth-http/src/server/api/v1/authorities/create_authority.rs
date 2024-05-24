use axum::{extract::State, response::IntoResponse, Json};
pub use oxidauth_kernel::authorities::create_authority::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthorityReq {
    pub authority: CreateAuthority,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthorityRes {
    pub authority: Authority,
}

#[tracing::instrument(name = "create_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(mut params): Json<CreateAuthorityReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!(
            "{:?} has {}",
            jwt.sub, PERMISSION
        ),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {}",
                jwt.sub, PERMISSION
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::fail().error(err.to_string()),
    }

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
