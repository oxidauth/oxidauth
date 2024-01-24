use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::role_permission_grants::delete_role_permission_grant::*;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type DeleteRolePermissionGrantReq = DeleteRolePermissionGrant;

pub type DeleteRolePermissionGrantRes = RolePermission;

#[tracing::instrument(
    name = "delete_role_permission_grant_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeleteRolePermissionGrantReq>,
) -> impl IntoResponse {
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
