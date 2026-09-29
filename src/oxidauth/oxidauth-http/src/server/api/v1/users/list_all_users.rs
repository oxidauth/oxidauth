use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::list_all_users::*;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type ListAllUsersReq = ListAllUsers;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllUsersRes {
    pub users: Vec<User>,
}

#[tracing::instrument(name = "list_all_users_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListAllUsersReq>,
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

    let service = provider.fetch::<ListAllUsersService>();

    info!("provided ListAllUsersService");

    let result = service.call(&params).await;

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

            Response::fail().error(err.into_error())
        },
    }
}
