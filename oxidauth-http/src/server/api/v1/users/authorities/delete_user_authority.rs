use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_authorities::delete_user_authority::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type DeleteUserAuthorityReq = DeleteUserAuthority;

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

    let result = service.call(&params).await;

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
