use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::permissions::create_permission::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct CreatePermissionReq {
    pub permission: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePermissionRes {
    pub permission: Permission,
}

#[tracing::instrument(name = "create_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<CreatePermissionReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<CreatePermissionService>();

    info!("provided CreatePermissionService");

    let result = service
        .call(&params.permission)
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

            Response::fail().error(err.into_error())
        },
    }
}
