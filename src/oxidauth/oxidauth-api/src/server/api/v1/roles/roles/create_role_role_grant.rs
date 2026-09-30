use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    roles::roles::create_role_role_grant::{CreateRoleRoleGrantReq, CreateRoleRoleGrantRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, role_role_grants::create_role_role_grant::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "create_role_role_grant_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<CreateRoleRoleGrantReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<CreateRoleRoleGrantService>();

    info!("provided CreateRoleRoleGrantService");

    let result = service
        .create_role_role_grant(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
