use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::find_user_by_username::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct FindUserByUsernameReq {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct FindUserByUsernameRes {
    pub user: User,
}

#[tracing::instrument(name = "find_user_by_username_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FindUserByUsernameReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<FindUserByUsernameService>();

    info!("provided FindUserByUsernameService");

    let result = service
        .call(params.username)
        .await;

    match result {
        Ok(user) => {
            info!(
                message = "successfully found user by username",
                user = ?user,
            );

            Response::success().payload(FindUserByUsernameRes { user })
        },
        Err(err) => {
            info!(
                message = "failed to find user by username",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
