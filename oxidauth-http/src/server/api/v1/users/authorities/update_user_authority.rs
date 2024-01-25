use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_authorities::update_user_authority::*;
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};
use uuid::Uuid;

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

use super::PERMISSION;

#[derive(Debug, Deserialize)]
pub struct UpdateUserAuthorityPathReq {
    user_id: Uuid,
    authority_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserAuthorityBodyReq {
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct UpdateUserAuthorityRes {
    pub user_authority: UserAuthority,
}

#[tracing::instrument(name = "update_user_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<UpdateUserAuthorityPathReq>,
    Json(request): Json<UpdateUserAuthorityBodyReq>,
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

    let service = provider.fetch::<UpdateUserAuthorityService>();

    info!("provided UpdateUserAuthorityService");

    let result = service
        .call(&UpdateUserAuthority {
            user_id: params.user_id,
            authority_id: params.authority_id,
            params: request.params,
        })
        .await;

    match result {
        Ok(user_authority) => {
            info!(
                message = "successfully updated user_authority",
                user_authority = ?user_authority,
            );

            Response::success()
                .payload(UpdateUserAuthorityRes { user_authority })
        },
        Err(err) => {
            info!(
                message = "failed to update user_authority",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
