use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::permissions::find_permission_by_parts::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type FindPermissionByPartsReq = FindPermissionByParts;

#[derive(Debug, Serialize)]
pub struct FindPermissionByPartsRes {
    pub permission: Permission,
}

#[tracing::instrument(name = "create_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FindPermissionByPartsReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<FindPermissionByPartsService>();

    info!("provided FindPermissionByPartsService");

    let result = service
        .call(&params)
        .await;

    match result {
        Ok(permission) => {
            info!(
                message = "successfully created permission",
                permission = ?permission,
            );

            Response::success().payload(FindPermissionByPartsRes { permission })
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

