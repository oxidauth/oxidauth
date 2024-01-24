use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::delete_user_by_id::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type DeleteUserByIdReq = DeleteUserById;

#[derive(Debug, Serialize)]
pub struct DeleteUserByIdRes {
    pub user: User,
}

#[tracing::instrument(name = "delete_user_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeleteUserByIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeleteUserByIdService>();

    info!("provided DeleteUserByIdService");

    let result = service.call(&params).await;

    match result {
        Ok(user) => {
            info!(
                message = "successfully deleted user by id",
                user = ?user,
            );

            Response::success().payload(DeleteUserByIdRes { user })
        },
        Err(err) => {
            info!(
                message = "failed to delete user by id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
