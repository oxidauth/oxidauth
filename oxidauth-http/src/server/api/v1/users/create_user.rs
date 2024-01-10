use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::users::create_user::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct CreateUserReq {
    pub user: CreateUser,
}

#[derive(Debug, Serialize)]
pub struct CreateUserRes {
    pub user: User,
}

#[tracing::instrument(name = "create_user_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<CreateUserReq>,
) -> impl IntoResponse {
    info!(
        message = "received create user request",
        params = ?params,
    );

    let service = provider.fetch::<CreateUserService>();

    let result = service
        .call(&params.user)
        .await;

    match result {
        Ok(user) => {
            info!(
                message = "successfully created user",
                user = ?user,
            );

            Response::success().payload(CreateUserRes { user })
        },
        Err(err) => {
            info!(
                message = "failed to create user",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
