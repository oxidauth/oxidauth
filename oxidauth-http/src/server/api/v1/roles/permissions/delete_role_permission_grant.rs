use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::role_permission_grants::delete_role_permission_grant::*;
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type DeleteRolePermissionGrantReq = DeleteRolePermissionGrant;

pub type DeleteRolePermissionGrantRes = RolePermission;

#[tracing::instrument(
    name = "delete_role_permission_grant_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteRolePermissionGrantReq>,
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

    let service = provider.fetch::<DeleteRolePermissionGrantService>();

    info!("provided DeleteRolePermissionGrantService");

    let result = service.call(&params).await;

    match result {
        Ok(res) => {
            info!(
                message = "successfully deleted role permission grant",
                res = ?res,
            );

            Response::success().payload(res)
        },
        Err(err) => {
            info!(
                message = "failed to delete role permission grant",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
