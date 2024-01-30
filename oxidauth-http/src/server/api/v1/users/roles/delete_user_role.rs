use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_role_grants::delete_user_role_grant::*;
use oxidauth_permission::parse_and_validate;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type DeleteUserRoleReq = DeleteUserRoleGrant;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserRoleRes {
    pub user_role: UserRole,
}

#[tracing::instrument(name = "delete_user_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteUserRoleReq>,
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

    let service = provider.fetch::<DeleteUserRoleGrantService>();

    info!("provided DeleteUserRoleGrantService");

    let result = service.call(&params).await;

    match result {
        Ok(user_role) => {
            info!(
                message = "successfully deleted user role",
                user_role = ?user_role,
            );

            Response::success().payload(DeleteUserRoleRes { user_role })
        },
        Err(err) => {
            info!(
                message = "failed to delete user role",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
