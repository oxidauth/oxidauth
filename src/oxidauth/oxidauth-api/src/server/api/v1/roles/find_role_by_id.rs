use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    roles::find_role_by_id::{FindRoleByIdReq, FindRoleByIdRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, roles::find_role_by_id::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "find_role_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindRoleByIdReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<FindRoleByIdService>();

    info!("provided FindRoleByIdService");

    let result = service
        .find_role_by_id(&params)
        .await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully found role by id",
                role = ?role,
            );

            Response::success().payload(FindRoleByIdRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to find role by id",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
