use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_authorities::list_user_authorities_by_user_id::*;
use oxidauth_permission::parse_and_validate;
use serde::Serialize;
use tracing::{info, warn};

use crate::response::Response;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

use super::PERMISSION;

pub type ListUserAuthoritiesByUserIdReq = ListUserAuthoritiesByUserId;

#[derive(Debug, Serialize)]
pub struct ListUserAuthoritiesByUserIdRes {
    pub user_authorities: Vec<UserAuthorityWithAuthority>,
}

#[tracing::instrument(
    name = "list_user_authorities_by_user_id_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListUserAuthoritiesByUserIdReq>,
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

    let service = provider.fetch::<ListUserAuthoritiesByUserIdService>();

    info!("provided ListUserAuthoritiesByUserIdService");

    let result = service.call(&params).await;

    match result {
        Ok(user_authorities) => {
            info!(
                message = "successfully listing user authorities by user_id",
                user_authorities = ?user_authorities,
            );

            Response::success()
                .payload(ListUserAuthoritiesByUserIdRes { user_authorities })
        },
        Err(err) => {
            info!(
                message = "failed to list user authorities by user_id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
