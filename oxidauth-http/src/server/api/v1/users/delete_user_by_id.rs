use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::delete_user_by_id::*;
use oxidauth_permission::parse_and_validate;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type DeleteUserByIdReq = DeleteUserById;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserByIdRes {
    pub user: User,
}

#[tracing::instrument(name = "delete_user_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteUserByIdReq>,
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
