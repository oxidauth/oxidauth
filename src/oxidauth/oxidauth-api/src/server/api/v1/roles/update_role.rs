use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    roles::update_role::{UpdateRolePathReq, UpdateRoleReq, UpdateRoleRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, roles::update_role::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "update_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<UpdateRolePathReq>,
    Json(body): Json<UpdateRoleReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<UpdateRoleService>();

    info!("provided UpdateRoleService");

    let mut updates = body.role;

    updates.role_id = Some(params.role_id);

    let result = service
        .update_role(&updates)
        .await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully updated role",
                role = ?role,
            );

            Response::success().payload(UpdateRoleRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to update role",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
