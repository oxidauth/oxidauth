use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_authorities::delete_user_authority::*;
use oxidauth_permission::parse_and_validate;
use serde::Serialize;
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type DeleteUserAuthorityReq = DeleteUserAuthority;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserAuthorityRes {
    pub user_authority: UserAuthority,
}

#[tracing::instrument(name = "delete_user_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteUserAuthorityReq>,
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

    let service = provider.fetch::<DeleteUserAuthorityService>();

    info!("provided DeleteUserAuthorityService");

    let result = service.call(&params).await;

    match result {
        Ok(user_authority) => {
            info!(
                message = "successfully deleted user_authority",
                user_authority = ?user_authority,
            );

            Response::success()
                .payload(DeleteUserAuthorityRes { user_authority })
        },
        Err(err) => {
            info!(
                message = "failed to delete user_authority",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
