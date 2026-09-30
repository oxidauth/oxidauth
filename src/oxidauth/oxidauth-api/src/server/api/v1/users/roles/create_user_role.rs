use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{Response, users::roles::create_user_role::CreateUserRoleReq};
use oxidauth_kernel::{error::IntoOxidAuthError, user_role_grants::create_user_role_grant::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "create_user_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<CreateUserRoleReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<CreateUserRoleGrantService>();

    info!("provided CreateUserRoleGrantService");

    let result = service
        .create_user_role_grant(&params)
        .await;

    match result {
        Ok(user_role) => {
            info!(
                message = "successfully created user role",
                user_role = ?user_role,
            );

            Response::success().payload(user_role)
        },
        Err(err) => {
            info!(
                message = "failed to create user role",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
