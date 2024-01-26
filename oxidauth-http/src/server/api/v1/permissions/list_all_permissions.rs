use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::permissions::list_all_permissions::*;
use oxidauth_permission::parse_and_validate;
use serde::Serialize;
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

type ListAllPermissionsReq = ListAllPermissions;

#[derive(Debug, Serialize)]
pub struct ListAllPermissionsRes {
    pub permissions: Vec<Permission>,
}

#[tracing::instrument(name = "list_all_permissions_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListAllPermissionsReq>,
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

    let service = provider.fetch::<ListAllPermissionsService>();

    info!("provided ListAllPermissionsService");

    let result = service.call(&params).await;

    match result {
        Ok(permissions) => {
            info!(
                message = "successfully listed all permission",
                permissions = ?permissions,
            );

            Response::success().payload(ListAllPermissionsRes { permissions })
        },
        Err(err) => {
            info!(
                message = "failed to list all permission",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
