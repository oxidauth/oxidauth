use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    users::update_user::{UpdateUserBodyReq, UpdateUserPathReq, UpdateUserRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, users::update_user::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "update_user_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(path): Path<UpdateUserPathReq>,
    Json(body): Json<UpdateUserBodyReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<UpdateUserService>();

    info!("provided UpdateUserService");

    let mut update_user = UpdateUser {
        id: path.user_id,
        username: body.user.username,
        email: body.user.email,
        first_name: body.user.first_name,
        last_name: body.user.last_name,
        status: body.user.status,
        profile: body.user.profile,
    };

    let result = service
        .update_user(&mut update_user)
        .await;

    match result {
        Ok(user) => {
            info!(
                message = "successfully updated user",
                user = ?user,
            );

            Response::success().payload(UpdateUserRes { user })
        },
        Err(err) => {
            info!(
                message = "failed to update user",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
