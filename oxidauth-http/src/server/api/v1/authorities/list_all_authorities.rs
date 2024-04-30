use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::authorities::list_all_authorities::*;
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

pub type ListAllAuthoritiesReq = ListAllAuthorities;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllAuthoritiesRes {
    pub authorities: Vec<Authority>,
}

#[tracing::instrument(name = "list_all_authorities_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListAllAuthoritiesReq>,
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

    let service = provider.fetch::<ListAllAuthoritiesService>();

    info!("provided ListAllAuthoritiesService");

    let result = service.call(&params).await;

    match result {
        Ok(authorities) => {
            info!(
                message = "successfully listed all authorities",
                authorities = ?authorities,
            );

            Response::success().payload(ListAllAuthoritiesRes { authorities })
        },
        Err(err) => {
            info!(
                message = "failed to list all authorities",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
