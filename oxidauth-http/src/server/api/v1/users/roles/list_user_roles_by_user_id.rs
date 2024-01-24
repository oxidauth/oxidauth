use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_role_grants::list_user_role_grants_by_user_id::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type ListUserRoleGrantsByUserIdReq = ListUserRoleGrantsByUserId;

#[derive(Debug, Serialize)]
pub struct ListUserRoleGrantsByUserIdRes {
    pub user_role_grants: Vec<UserRole>,
}

#[tracing::instrument(
    name = "list_user_role_grants_by_user_id_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListUserRoleGrantsByUserIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListUserRoleGrantsByUserIdService>();

    info!("provided ListUserRoleGrantsByUserIdService");

    let result = service.call(&params).await;

    match result {
        Ok(user_role_grants) => {
            info!(
                message = "successfully listing user role grants by user_id",
                user_role_grants = ?user_role_grants,
            );

            Response::success()
                .payload(ListUserRoleGrantsByUserIdRes { user_role_grants })
        },
        Err(err) => {
            info!(
                message = "failed to list user role grants by user_id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
