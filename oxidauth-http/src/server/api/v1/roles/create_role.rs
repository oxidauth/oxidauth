use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::roles::create_role::*;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleReq {
    pub role: CreateRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRes {
    pub role: Role,
}

#[tracing::instrument(name = "create_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<CreateRoleReq>,
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

    let service = provider.fetch::<CreateRoleService>();

    info!("provided CreateRoleService");

    let result = service
        .call(&params.role)
        .await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully created role",
                role = ?role,
            );

            Response::success().payload(CreateRoleRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to create role",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
