use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_permission_grants::list_user_permission_grants_by_user_id::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type ListUserPermissionGrantsByUserIdReq = ListUserPermissionGrantsByUserId;

#[derive(Debug, Serialize)]
pub struct ListUserPermissionGrantsByUserIdRes {
    pub user_permission_grants: Vec<UserPermission>,
}

#[tracing::instrument(
    name = "list_user_permission_grants_by_user_id_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListUserPermissionGrantsByUserIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListUserPermissionGrantsByUserIdService>();

    info!("provided ListUserPermissionGrantsByUserIdService");

    let result = service.call(&params).await;

    match result {
        Ok(user_permission_grants) => {
            info!(
                message = "successfully listing user permission grants by user_id",
                user_permission_grants = ?user_permission_grants,
            );

            Response::success().payload(
                ListUserPermissionGrantsByUserIdRes {
                    user_permission_grants,
                },
            )
        },
        Err(err) => {
            info!(
                message = "failed to list user permission grants by user_id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
