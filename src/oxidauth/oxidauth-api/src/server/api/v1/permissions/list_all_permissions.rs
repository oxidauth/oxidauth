use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    permissions::list_all_permissions::{ListAllPermissionsReq, ListAllPermissionsRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, permissions::list_all_permissions::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "list_all_permissions_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListAllPermissionsReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<ListAllPermissionsService>();

    info!("provided ListAllPermissionsService");

    let result = service
        .list_all_permissions(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
