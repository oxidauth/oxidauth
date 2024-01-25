use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::create_user::*;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

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
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<CreateUserReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!(
            "{:?} has {}",
            jwt.sub, PERMISSION
        ),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {}",
                jwt.sub, PERMISSION
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::fail().error(err.to_string()),
    }

    let service = provider.fetch::<CreateUserService>();

    info!("provided CreateUserService");

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
