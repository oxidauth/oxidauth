use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::permissions::delete_permission::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type DeletePermissionReq = DeletePermission;

#[derive(Debug, Serialize)]
pub struct DeletePermissionRes {
    pub permission: Permission,
}

#[tracing::instrument(name = "delete_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeletePermissionReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeletePermissionService>();

    info!("provided DeletePermissionService");

    let result = service
        .call(&params)
        .await;

    match result {
        Ok(permission) => {
            info!(
                message = "successfully deleted permission",
                permission = ?permission,
            );

            Response::success().payload(DeletePermissionRes { permission })
        },
        Err(err) => {
            info!(
                message = "failed to delete permission",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
