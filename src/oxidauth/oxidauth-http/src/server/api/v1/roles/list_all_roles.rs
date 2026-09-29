use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::roles::list_all_roles::*;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type ListAllRolesReq = ListAllRoles;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllRolesRes {
    pub roles: Vec<Role>,
}

#[tracing::instrument(name = "list_all_roles_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListAllRolesReq>,
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

    let service = provider.fetch::<ListAllRolesService>();

    info!("provided ListAllRolesService");

    let result = service.call(&params).await;

    match result {
        Ok(roles) => {
            info!(
                message = "successfully listed roles",
                roles = ?roles,
            );

            Response::success().payload(ListAllRolesRes { roles })
        },
        Err(err) => {
            info!(
                message = "failed to list roles",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
