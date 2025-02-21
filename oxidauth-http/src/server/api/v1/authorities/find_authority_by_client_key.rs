use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use oxidauth_kernel::{
    authorities::find_authority_by_client_key::*, error::IntoOxidAuthError,
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type FindAuthorityByClientKeyReq = FindAuthorityByClientKey;

pub type FindAuthorityByClientKeyRes = FindAuthorityByClientKeyResult;

#[tracing::instrument(
    name = "find_authority_by_client_key_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindAuthorityByClientKeyReq>,
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

    let service = provider.fetch::<FindAuthorityByClientKeyService>();

    info!("provided FindAuthorityByClientKeyService");

    let result = service.call(&params).await;

    match result {
        Ok(authority) => {
            info!(
                message = "successfully found authority by client_key",
                authority = ?authority,
            );

            Response::success()
                .payload(FindAuthorityByClientKeyRes { authority })
        },
        Err(err) => {
            info!(
                message = "failed to find authority by client_key",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
