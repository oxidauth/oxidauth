use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::permissions::delete_permission::*;
use oxidauth_permission::parse_and_validate;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type DeletePermissionReq = DeletePermission;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePermissionRes {
    pub permission: Permission,
}

#[tracing::instrument(name = "delete_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeletePermissionReq>,
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

    let service = provider.fetch::<DeletePermissionService>();

    info!("provided DeletePermissionService");

    let result = service.call(&params).await;

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

            Response::fail().error(err.into_error())
        },
    }
}
