use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::update_user::*;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserBodyReq {
    pub user: UpdateUserUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<UserStatus>,
    pub profile: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRes {
    pub user: User,
}

#[tracing::instrument(name = "update_user_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(path): Path<UpdateUserPathReq>,
    Json(body): Json<UpdateUserBodyReq>,
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
        .call(&mut update_user)
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
