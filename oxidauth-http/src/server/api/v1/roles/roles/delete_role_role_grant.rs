use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::role_role_grants::delete_role_role_grant::*;
use oxidauth_permission::parse_and_validate;
use serde::Serialize;
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type DeleteRoleRoleGrantReq = DeleteRoleRoleGrant;

#[derive(Debug, Serialize)]
pub struct DeleteRoleRoleGrantRes {
    pub grant: RoleRoleGrant,
}

#[tracing::instrument(name = "delete_role_role_grant_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteRoleRoleGrantReq>,
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

    let service = provider.fetch::<DeleteRoleRoleGrantService>();

    info!("provided DeleteRoleRoleGrantService");

    let result = service.call(&params).await;

    match result {
        Ok(grant) => {
            info!(
                message = "successfully deleted role role grant",
                grant = ?grant,
            );

            Response::success().payload(DeleteRoleRoleGrantRes { grant })
        },
        Err(err) => {
            info!(
                message = "failed to delete role role grant",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
