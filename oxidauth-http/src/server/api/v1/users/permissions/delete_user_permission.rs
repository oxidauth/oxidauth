use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_permission_grants::delete_user_permission_grant::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct DeleteUserPermissionReq {
    pub user_id: Uuid,
    pub permission: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteUserPermissionRes {
    pub user_permission: UserPermission,
}

#[tracing::instrument(name = "delete_user_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeleteUserPermissionReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeleteUserPermissionGrantService>();

    info!("provided DeleteUserPermissionGrantService");

    let result = service
        .call(&DeleteUserPermission {
            user_id: params.user_id,
            permission: params.permission,
        })
        .await;

    match result {
        Ok(user_permission) => {
            info!(
                message = "successfully deleted user permission",
                user_permission = ?user_permission,
            );

            Response::success()
                .payload(DeleteUserPermissionRes { user_permission })
        },
        Err(err) => {
            info!(
                message = "failed to delete user permission",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
