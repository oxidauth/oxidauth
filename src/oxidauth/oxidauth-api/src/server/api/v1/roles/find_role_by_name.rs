use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    roles::find_role_by_name::{FindRoleByNameReq, FindRoleByNameRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, roles::find_role_by_name::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "find_role_by_name_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindRoleByNameReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<FindRoleByNameService>();

    info!("provided FindRoleByNameService");

    let result = service
        .find_role_by_name(&params)
        .await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully found role by name",
                role = ?role,
            );

            Response::success().payload(FindRoleByNameRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to find role by name",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
