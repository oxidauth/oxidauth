use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{
    Response,
    roles::create_role::{CreateRoleReq, CreateRoleRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, roles::create_role::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "create_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<CreateRoleReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<CreateRoleService>();

    info!("provided CreateRoleService");

    let result = service
        .create_role(&params.role)
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

            Response::bad_request().error(err.into_error())
        },
    }
}
