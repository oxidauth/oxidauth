use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_permission_grants::create_user_permission_grant::*;
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type CreateUserPermissionReq = CreateUserPermission;

pub type CreateUserPermissionRes = UserPermission;

#[tracing::instrument(name = "create_user_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<CreateUserPermissionReq>,
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

    let service = provider.fetch::<CreateUserPermissionGrantService>();

    info!("provided CreateUserPermissionGrantService");

    let result = service.call(&params).await;

    match result {
        Ok(user_permission) => {
            info!(
                message = "successfully created user permission",
                user_permission = ?user_permission,
            );

            Response::success().payload(user_permission)
        },
        Err(err) => {
            info!(
                message = "failed to create user permission",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
