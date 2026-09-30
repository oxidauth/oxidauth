use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    permissions::find_permission_by_parts::{FindPermissionByPartsReq, FindPermissionByPartsRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, permissions::find_permission_by_parts::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "find_permission_by_parts_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindPermissionByPartsReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<FindPermissionByPartsService>();

    info!("provided FindPermissionByPartsService");

    let result = service
        .find_permission_by_parts(&params)
        .await;

    match result {
        Ok(permission) => {
            info!(
                message = "successfully found permission by parts",
                permission = ?permission,
            );

            Response::success().payload(FindPermissionByPartsRes { permission })
        },
        Err(err) => {
            info!(
                message = "failed to find permission by parts",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
