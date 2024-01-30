use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_permission_grants::list_user_permission_grants_by_user_id::*;
use oxidauth_permission::parse_and_validate;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type ListUserPermissionGrantsByUserIdReq = ListUserPermissionGrantsByUserId;

#[derive(Debug, Deserialize, Serialize)]
pub struct ListUserPermissionGrantsByUserIdRes {
    pub user_permission_grants: Vec<UserPermission>,
}

#[tracing::instrument(
    name = "list_user_permission_grants_by_user_id_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListUserPermissionGrantsByUserIdReq>,
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

    let service = provider.fetch::<ListUserPermissionGrantsByUserIdService>();

    info!("provided ListUserPermissionGrantsByUserIdService");

    let result = service.call(&params).await;

    match result {
        Ok(user_permission_grants) => {
            info!(
                message = "successfully listing user permission grants by user_id",
                user_permission_grants = ?user_permission_grants,
            );

            Response::success().payload(
                ListUserPermissionGrantsByUserIdRes {
                    user_permission_grants,
                },
            )
        },
        Err(err) => {
            info!(
                message = "failed to list user permission grants by user_id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
