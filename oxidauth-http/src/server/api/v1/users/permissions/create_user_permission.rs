use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_permission_grants::create_user_permission_grant::*;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type CreateUserPermissionReq = CreateUserPermission;

pub type CreateUserPermissionRes = UserPermission;

#[tracing::instrument(name = "create_user_permission_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<CreateUserPermissionReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<CreateUserPermissionGrantService>();

    info!("provided CreateUserPermissionGrantService");

    let result = service.call(&params).await;

    match result {
        Ok(user_permission) => {
            info!(
                message = "successfully created user permission",
                user_permission = ?user_permission,
            );

            Response::success().payload(user_permission)
        },
        Err(err) => {
            info!(
                message = "failed to create user permission",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
