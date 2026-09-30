use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    users::authorities::create_user_authority::{
        CreateUserAuthorityBodyReq,
        CreateUserAuthorityPathReq,
        CreateUserAuthorityRes,
    },
};
use oxidauth_kernel::{error::IntoOxidAuthError, user_authorities::create_user_authority::*};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "create_user_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<CreateUserAuthorityPathReq>,
    Json(request): Json<CreateUserAuthorityBodyReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<CreateUserAuthorityService>();

    info!("provided CreateUserAuthorityService");

    let result = service
        .create_user_authority(&CreateUserAuthorityParams {
            user_id: params.user_id,
            client_key: request.client_key,
            params: request.user_authority.params,
        })
        .await;

    match result {
        Ok(user_authority) => {
            info!(
                message = "successfully created user_authority",
                user_authority = ?user_authority,
            );

            Response::success().payload(CreateUserAuthorityRes { user_authority })
        },
        Err(err) => {
            info!(
                message = "failed to create user_authority",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
