use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::roles::delete_role::*;
use oxidauth_permission::parse_and_validate;
use serde::Serialize;
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type DeleteRoleReq = DeleteRole;

#[derive(Debug, Serialize)]
pub struct DeleteRoleRes {
    pub role: Role,
}

#[tracing::instrument(name = "delete_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteRoleReq>,
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

    let service = provider.fetch::<DeleteRoleService>();

    info!("provided DeleteRoleService");

    let result = service.call(&params).await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully deleted role",
                role = ?role,
            );

            Response::success().payload(DeleteRoleRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to delete role",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
