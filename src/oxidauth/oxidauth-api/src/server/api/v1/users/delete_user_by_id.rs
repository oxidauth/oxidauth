use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    users::delete_user_by_id::{DeleteUserByIdReq, DeleteUserByIdRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, users::delete_user_by_id::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "delete_user_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteUserByIdReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<DeleteUserByIdService>();

    info!("provided DeleteUserByIdService");

    let result = service
        .delete_user_by_id(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
