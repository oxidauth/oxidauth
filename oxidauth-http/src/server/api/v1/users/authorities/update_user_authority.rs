use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_authorities::update_user_authority::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

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
    Path(params): Path<UpdateUserAuthorityPathReq>,
    Json(request): Json<UpdateUserAuthorityBodyReq>,
) -> impl IntoResponse {
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
