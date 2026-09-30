use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    users::permissions::delete_user_permission::{
        DeleteUserPermissionReq,
        DeleteUserPermissionRes,
    },
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    user_permission_grants::delete_user_permission_grant::*,
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "delete_user_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteUserPermissionReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<DeleteUserPermissionGrantService>();

    info!("provided DeleteUserPermissionGrantService");

    let result = service
        .delete_user_permission_grant(&params)
        .await;

    match result {
        Ok(user_permission) => {
            info!(
                message = "successfully deleted user permission",
                user_permission = ?user_permission,
            );

            Response::success().payload(DeleteUserPermissionRes { user_permission })
        },
        Err(err) => {
            info!(
                message = "failed to delete user permission",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
