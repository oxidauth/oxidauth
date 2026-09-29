use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::roles::find_role_by_id::*;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type FindRoleByIdReq = FindRoleById;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRoleByIdRes {
    pub role: Role,
}

#[tracing::instrument(name = "find_role_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindRoleByIdReq>,
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

    let service = provider.fetch::<FindRoleByIdService>();

    info!("provided FindRoleByIdService");

    let result = service.call(&params).await;

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

            Response::fail().error(err.into_error())
        },
    }
}
