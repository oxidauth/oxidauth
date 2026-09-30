use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    permissions::delete_permission::{DeletePermissionReq, DeletePermissionRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, permissions::delete_permission::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "delete_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeletePermissionReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<DeletePermissionService>();

    info!("provided DeletePermissionService");

    let result = service
        .delete_permission(&params)
        .await;

    match result {
        Ok(permission) => {
            info!(
                message = "successfully deleted permission",
                permission = ?permission,
            );

            Response::success().payload(DeletePermissionRes { permission })
        },
        Err(err) => {
            info!(
                message = "failed to delete permission",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
