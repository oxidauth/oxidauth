use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use oxidauth_kernel::authorities::update_authority::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAuthorityPathReq {
    pub authority_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAuthorityReq {
    pub authority: UpdateAuthority,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAuthorityRes {
    pub authority: Authority,
}

#[tracing::instrument(name = "update_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(path): Path<UpdateAuthorityPathReq>,
    Json(mut params): Json<UpdateAuthorityReq>,
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

    let service = provider.fetch::<UpdateAuthorityService>();

    info!("provided UpdateAuthorityService");

    params.authority.id = Some(path.authority_id);

    let result = service
        .call(&mut params.authority)
        .await;

    match result {
        Ok(authority) => {
            info!(
            message = "successfully updated authority",
            authority = ?authority,
            );

            Response::success().payload(UpdateAuthorityRes { authority })
        },
        Err(err) => {
            info!(
            message = "failed to update authority",
            err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
