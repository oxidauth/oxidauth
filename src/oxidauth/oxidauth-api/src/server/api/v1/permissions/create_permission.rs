use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    permissions::create_permission::{CreatePermissionReq, CreatePermissionRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, permissions::create_permission::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "create_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<CreatePermissionReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<CreatePermissionService>();

    info!("provided CreatePermissionService");

    let result = service
        .create_permission(&params)
        .await;

    match result {
        Ok(permission) => {
            info!(
                message = "successfully created permission",
                permission = ?permission,
            );

            Response::success().payload(CreatePermissionRes { permission })
        },
        Err(err) => {
            info!(
                message = "failed to create permission",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
