use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{
    Response,
    users::create_user::{CreateUserReq, CreateUserRes},
};
use oxidauth_kernel::error::IntoOxidAuthError;
pub use oxidauth_kernel::users::create_user::*;
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "create_user_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<CreateUserReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<CreateUserService>();

    info!("provided CreateUserService");

    let result = service
        .create_user(&params.user)
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

            Response::bad_request().error(err.into_error())
        },
    }
}
