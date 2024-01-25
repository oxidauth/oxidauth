use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::update_user::*;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

#[derive(Debug, Deserialize)]
pub struct UpdateUserPathReq {
    user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserBodyReq {
    pub user: UpdateUser,
}

#[derive(Debug, Serialize)]
pub struct UpdateUserRes {
    pub user: User,
}

#[tracing::instrument(name = "update_user_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<UpdateUserPathReq>,
    Json(request): Json<UpdateUserBodyReq>,
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

    let service = provider.fetch::<UpdateUserService>();

    info!("provided UpdateUserService");

    let mut updates = request.user;

    updates.id = Some(params.user_id);

    let result = service
        .call(&mut updates)
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

            Response::fail().error(err.into_error())
        },
    }
}
