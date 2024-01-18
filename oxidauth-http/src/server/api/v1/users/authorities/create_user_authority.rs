use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_authorities::create_user_authority::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct CreateUserAuthorityPathReq {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserAuthorityBodyReq {
    pub user_authority: CreateUserAuthority,
}

#[derive(Debug, Serialize)]
pub struct CreateUserAuthorityRes {
    pub user_authority: UserAuthority,
}

#[tracing::instrument(name = "create_user_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<CreateUserAuthorityPathReq>,
    Json(request): Json<CreateUserAuthorityBodyReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<CreateUserAuthorityService>();

    info!("provided CreateUserAuthorityService");

    let mut user_authority = request.user_authority;

    user_authority.user_id = Some(params.user_id);

    let result = service
        .call(&user_authority)
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
