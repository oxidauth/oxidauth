use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{Response, authorities::delete_authority::DeleteAuthorityRes};
use oxidauth_kernel::{authorities::delete_authority::*, error::IntoOxidAuthError};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

type DeleteAuthorityReq = DeleteAuthority;

#[tracing::instrument(name = "delete_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteAuthorityReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<DeleteAuthorityService>();

    info!("provided DeleteAuthorityService");

    let result = service
        .delete_authority(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
