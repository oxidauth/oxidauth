use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    roles::delete_role::{DeleteRoleReq, DeleteRoleRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, roles::delete_role::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "delete_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<DeleteRoleReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<DeleteRoleService>();

    info!("provided DeleteRoleService");

    let result = service
        .delete_role(&params)
        .await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully deleted role",
                role = ?role,
            );

            Response::success().payload(DeleteRoleRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to delete role",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
