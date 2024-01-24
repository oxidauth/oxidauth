use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::list_all_users::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type ListAllUsersReq = ListAllUsers;

#[derive(Debug, Serialize)]
pub struct ListAllUsersRes {
    pub users: Vec<User>,
}

#[tracing::instrument(name = "list_all_users_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListAllUsersReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListAllUsersService>();

    info!("provided ListAllUsersService");

    let result = service.call(&params).await;

    match result {
        Ok(users) => {
            info!(
                message = "successfully listing all users",
                users = ?users,
            );

            Response::success().payload(ListAllUsersRes { users })
        },
        Err(err) => {
            info!(
                message = "failed to list all users",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
