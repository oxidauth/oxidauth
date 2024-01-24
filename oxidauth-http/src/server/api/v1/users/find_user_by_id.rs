use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::users::find_user_by_id::*;
use oxidauth_permission::parse_and_validate_multiple;
use serde::Serialize;
use tracing::{info, warn};

use crate::response::Response;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

pub type FindUserByIdReq = FindUserById;

#[derive(Debug, Serialize)]
pub struct FindUserByIdRes {
    pub user: User,
}

#[tracing::instrument(name = "find_user_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindUserByIdReq>,
) -> impl IntoResponse {
    let mut challenges = vec!["oxidauth:users:manage".to_string()];

    if let Some(user_id) = jwt.sub {
        challenges.push(format!(
            "oxidauth:users.{}:read",
            user_id
        ));
    }

    match parse_and_validate_multiple(&challenges, &permissions) {
        Ok(true) => info!(
            "{:?} has {:?}",
            jwt.sub, challenges
        ),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {:?}",
                jwt.sub, challenges
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::fail().error(err.to_string()),
    }

    let service = provider.fetch::<FindUserByIdService>();

    info!("provided FindUserByIdService");

    let result = service.call(&params).await;

    match result {
        Ok(user) => {
            info!(
                message = "successfully found user by id",
                user = ?user,
            );

            Response::success().payload(FindUserByIdRes { user })
        },
        Err(err) => {
            info!(
                message = "failed to find user by id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
