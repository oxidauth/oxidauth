use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    users::list_all_users::{ListAllUsersReq, ListAllUsersRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, users::list_all_users::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "list_all_users_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListAllUsersReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<ListAllUsersService>();

    info!("provided ListAllUsersService");

    let result = service
        .list_all_users(&params)
        .await;

    match result {
        Ok(users) => {
            info!(
                message = "successfully listing all users",
                users = ?users,
            );

            Response::success().payload(ListAllUsersRes { users })
        },
        Err(err) => {
            info!(
                message = "failed to list all users",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
