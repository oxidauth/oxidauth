use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::permissions::list_all_permissions::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type ListAllPermissionsReq = ListAllPermissions;

#[derive(Debug, Serialize)]
pub struct ListAllPermissionsRes {
    pub permissions: Vec<Permission>,
}

#[tracing::instrument(name = "list_all_permissions_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListAllPermissionsReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListAllPermissionsService>();

    info!("provided ListAllPermissionsService");

    let result = service
        .call(&params)
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

            Response::fail().error(err.into_error())
        },
    }
}

