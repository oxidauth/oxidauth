use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    roles::permissions::list_role_permission_grants_by_role_id::{
        ListRolePermissionGrantsByRoleIdReq,
        ListRolePermissionGrantsByRoleIdRes,
    },
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    role_permission_grants::list_role_permission_grants_by_role_id::*,
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

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
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<ListRolePermissionGrantsByRoleIdService>();

    info!("provided ListRolePermissionGrantsByRoleIdService");

    let result = service
        .list_role_permission_grants_by_role_id(&params)
        .await;

    match result {
        Ok(permissions) => {
            info!(
                message = "successfully listed role permission grants by role_id",
                permissions = ?permissions,
            );

            Response::success().payload(ListRolePermissionGrantsByRoleIdRes { permissions })
        },
        Err(err) => {
            info!(
                message = "failed to list role permission grants by role_id",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
