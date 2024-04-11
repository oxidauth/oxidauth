use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::users::find_users_by_ids::*;
use oxidauth_kernel::{error::IntoOxidAuthError, users::User};
use oxidauth_permission::parse_and_validate_multiple;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

pub type FindUsersByIdsReq = FindUsersByIds;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUsersByIdsRes {
    pub users: Vec<User>,
    pub user_ids_not_found: Vec<Uuid>,
}

const CHALLENGES: [&str; 1] = ["oxidauth:users:manage"];

#[tracing::instrument(name = "find_users_by_ids_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<FindUsersByIdsReq>,
) -> impl IntoResponse {
    match parse_and_validate_multiple(&CHALLENGES, &permissions) {
        Ok(true) => info!(
            "{:?} has {:?}",
            jwt.sub, CHALLENGES
        ),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {:?}",
                jwt.sub, CHALLENGES
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::fail().error(err.to_string()),
    }

    let service = provider.fetch::<FindUsersByIdsService>();

    info!("provided FindUsersByIdsService");

    let result = service.call(&params).await;

    match result {
        Ok(UsersByIds {
            users,
            user_ids_not_found,
        }) => {
            info!(
                message = "successfully found users by ids",
                users = ?users,
                user_ids_not_found = ?user_ids_not_found,
            );

            Response::success().payload(FindUsersByIdsRes {
                users,
                user_ids_not_found,
            })
        },
        Err(err) => {
            info!(
                message = "failed to find users by ids",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
