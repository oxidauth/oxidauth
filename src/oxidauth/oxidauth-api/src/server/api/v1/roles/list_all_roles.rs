use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    roles::list_all_roles::{ListAllRolesReq, ListAllRolesRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, roles::list_all_roles::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "list_all_roles_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListAllRolesReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<ListAllRolesService>();

    info!("provided ListAllRolesService");

    let result = service
        .list_all_roles(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
