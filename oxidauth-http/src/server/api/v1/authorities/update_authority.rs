use axum::{extract::{Path, State}, response::IntoResponse, Json};
use oxidauth_kernel::authorities::update_authority::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct UpdateAuthorityPathReq {
    pub authority_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAuthorityReq {
    pub authority: UpdateAuthority,
}

#[derive(Debug, Serialize)]
pub struct UpdateAuthorityRes {
    pub authority: Authority,
}

#[tracing::instrument(name = "update_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(path): Path<UpdateAuthorityPathReq>,
    Json(mut params): Json<UpdateAuthorityReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<UpdateAuthorityService>();

    info!("provided UpdateAuthorityService");

    params.authority.id = Some(path.authority_id);

    let result = service
        .call(&mut params.authority)
        .await;

    match result {
        Ok(authority) => {
            info!(
                message = "successfully updated authority",
                authority = ?authority,
                );

            Response::success().payload(UpdateAuthorityRes { authority })
        },
        Err(err) => {
            info!(
                message = "failed to update authority",
                err = ?err,
                );

            Response::fail().error(err.into_error())
        },
    }
}
