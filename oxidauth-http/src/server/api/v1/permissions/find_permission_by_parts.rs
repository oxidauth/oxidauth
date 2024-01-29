use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::permissions::find_permission_by_parts::*;
use oxidauth_permission::parse_and_validate;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type FindPermissionByPartsReq = FindPermissionByParts;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPermissionByPartsRes {
    pub permission: Permission,
}

#[tracing::instrument(
    name = "find_permission_by_parts_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindPermissionByPartsReq>,
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

    let service = provider.fetch::<FindPermissionByPartsService>();

    info!("provided FindPermissionByPartsService");

    let result = service.call(&params).await;

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

            Response::fail().error(err.into_error())
        },
    }
}
