use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    roles::permissions::delete_role_permission_grant::DeleteRolePermissionGrantReq,
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    role_permission_grants::delete_role_permission_grant::*,
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "delete_role_permission_grant_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteRolePermissionGrantReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<DeleteRolePermissionGrantService>();

    info!("provided DeleteRolePermissionGrantService");

    let result = service
        .delete_role_permission_grant(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
