use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::role_role_grants::list_role_role_grants_by_child_id::*;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

pub type ListRoleRoleGrantsByChildIdReq = ListRoleRoleGrantsByChildId;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRoleRoleGrantsByChildIdRes {
    pub roles: Vec<RoleRoleGrantDetail>,
}

#[tracing::instrument(
    name = "list_role_role_grants_by_child_id_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListRoleRoleGrantsByChildIdReq>,
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

    let service = provider.fetch::<ListRoleRoleGrantsByChildIdService>();

    info!("provided ListRoleRoleGrantsByChildIdService");

    let result = service.call(&params).await;

    match result {
        Ok(roles) => {
            info!(
                message = "successfully listed role role grants by child_id",
                roles = ?roles,
            );

            Response::success()
                .payload(ListRoleRoleGrantsByChildIdRes { roles })
        },
        Err(err) => {
            info!(
                message = "failed to list role role grants by child_id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
