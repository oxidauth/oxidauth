use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::role_role_grants::create_role_role_grant::*;
use oxidauth_kernel::{error::IntoOxidAuthError, roles::Role};
use oxidauth_permission::parse_and_validate;
use serde::Serialize;
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type CreateRoleRoleGrantReq = CreateRoleRoleGrant;

#[derive(Debug, Serialize)]
pub struct CreateRoleRoleGrantRes {
    pub child: Role,
    pub grant: RoleRoleGrant,
}

#[tracing::instrument(name = "create_role_role_grant_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<CreateRoleRoleGrantReq>,
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

    let service = provider.fetch::<CreateRoleRoleGrantService>();

    info!("provided CreateRoleRoleGrantService");

    let result = service.call(&params).await;

    match result {
        Ok(res) => {
            info!(
                message = "successfully created role role grant",
                res = ?res,
            );

            Response::success().payload(CreateRoleRoleGrantRes {
                child: res.role,
                grant: res.grant,
            })
        },
        Err(err) => {
            info!(
                message = "failed to create role role grant",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
