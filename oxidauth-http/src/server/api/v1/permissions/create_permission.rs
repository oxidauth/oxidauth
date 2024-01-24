use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::permissions::create_permission::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type CreatePermissionReq = CreatePermission;

#[derive(Debug, Serialize)]
pub struct CreatePermissionRes {
    pub permission: Permission,
}

#[tracing::instrument(name = "create_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<CreatePermissionReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<CreatePermissionService>();

    info!("provided CreatePermissionService");

    let result = service.call(&params).await;

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

            Response::fail().error(err.into_error())
        },
    }
}
