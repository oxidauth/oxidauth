use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_permission::parse_and_validate;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type ListRolePermissionGrantsByRoleIdReq = ListRolePermissionGrantsByRoleId;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRolePermissionGrantsByRoleIdRes {
    pub permissions: Vec<RolePermission>,
}

#[tracing::instrument(
    name = "list_role_permission_grants_by_role_id_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListRolePermissionGrantsByRoleIdReq>,
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

    let service = provider.fetch::<ListRolePermissionGrantsByRoleIdService>();

    info!("provided ListRolePermissionGrantsByRoleIdService");

    let result = service.call(&params).await;

    match result {
        Ok(permissions) => {
            info!(
                message = "successfully listed role permission grants by role_id",
                permissions = ?permissions,
            );

            Response::success()
                .payload(ListRolePermissionGrantsByRoleIdRes { permissions })
        },
        Err(err) => {
            info!(
                message = "failed to list role permission grants by role_id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
