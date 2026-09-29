use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use oxidauth_kernel::{
    authorities::delete_authority::*, error::IntoOxidAuthError,
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

type DeleteAuthorityReq = DeleteAuthority;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteAuthorityRes {
    pub authority: Authority,
}

#[tracing::instrument(name = "delete_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteAuthorityReq>,
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

    let service = provider.fetch::<DeleteAuthorityService>();

    info!("provided DeleteAuthorityService");

    let result = service.call(&params).await;

    match result {
        Ok(authority) => {
            info!(
                message = "successfully deleted authority",
                authority = ?authority,
            );

            Response::success().payload(DeleteAuthorityRes { authority })
        },
        Err(err) => {
            info!(
                message = "failed to delete authority",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
