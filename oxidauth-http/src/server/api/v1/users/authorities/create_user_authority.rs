use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use oxidauth_kernel::{
    authorities::AuthorityStrategy, error::IntoOxidAuthError,
};
use oxidauth_kernel::{user_authorities::create_user_authority::*, JsonValue};
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};

use tracing::{info, warn};
use uuid::Uuid;

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserAuthorityPathReq {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserAuthorityBodyReq {
    pub authority_strategy: AuthorityStrategy,
    pub user_authority: UserAuthorityParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthorityParams {
    pub params: JsonValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserAuthorityRes {
    pub user_authority: UserAuthority,
}

#[tracing::instrument(name = "create_user_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<CreateUserAuthorityPathReq>,
    Json(request): Json<CreateUserAuthorityBodyReq>,
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

    let service = provider.fetch::<CreateUserAuthorityService>();

    info!("provided CreateUserAuthorityService");

    let result = service
        .call(&CreateUserAuthorityParams {
            user_id: params.user_id,
            strategy: request.authority_strategy,
            params: request.user_authority.params,
        })
        .await;

    match result {
        Ok(user_authority) => {
            info!(
                message = "successfully created user_authority",
                user_authority = ?user_authority,
            );

            Response::success()
                .payload(CreateUserAuthorityRes { user_authority })
        },
        Err(err) => {
            info!(
                message = "failed to create user_authority",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
