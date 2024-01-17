use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_role_grants::delete_user_role_grant::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct DeleteUserRoleReq {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct DeleteUserRoleRes {
    pub user_role: UserRole,
}

#[tracing::instrument(name = "delete_user_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeleteUserRoleReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeleteUserRoleGrantService>();

    info!("provided DeleteUserRoleGrantService");

    let result = service
        .call(&DeleteUserRoleGrant {
            user_id: params.user_id,
            role_id: params.role_id,
        })
        .await;

    match result {
        Ok(user_role) => {
            info!(
                message = "successfully deleted user role",
                user_role = ?user_role,
            );

            Response::success().payload(DeleteUserRoleRes { user_role })
        },
        Err(err) => {
            info!(
                message = "failed to delete user role",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
