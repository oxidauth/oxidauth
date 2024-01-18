use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_authorities::delete_user_authority::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct DeleteUserAuthorityReq {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct DeleteUserAuthorityRes {
    pub user_authority: UserAuthority,
}

#[tracing::instrument(name = "delete_user_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeleteUserAuthorityReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeleteUserAuthorityService>();

    info!("provided DeleteUserAuthorityService");

    let result = service
        .call(&DeleteUserAuthority {
            user_id: params.user_id,
            authority_id: params.authority_id,
        })
        .await;

    match result {
        Ok(user_authority) => {
            info!(
                message = "successfully deleted user_authority",
                user_authority = ?user_authority,
            );

            Response::success()
                .payload(DeleteUserAuthorityRes { user_authority })
        },
        Err(err) => {
            info!(
                message = "failed to delete user_authority",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
