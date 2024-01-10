use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::find_user_by_id::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct FindUserByIdReq {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct FindUserByIdRes {
    pub user: User,
}

#[tracing::instrument(name = "find_user_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FindUserByIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<FindUserByIdService>();

    info!("provided FindUserByIdService");

    let result = service
        .call(params.user_id)
        .await;

    match result {
        Ok(user) => {
            info!(
                message = "successfully found user by id",
                user = ?user,
            );

            Response::success().payload(FindUserByIdRes { user })
        },
        Err(err) => {
            info!(
                message = "failed to find user by id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
