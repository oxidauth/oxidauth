use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::authorities::delete_authority::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type DeleteAuthorityReq = DeleteAuthority;

#[derive(Debug, Serialize)]
pub struct DeleteAuthorityRes {
    pub authority: Authority,
}

#[tracing::instrument(name = "delete_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeleteAuthorityReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeleteAuthorityService>();

    info!("provided DeleteAuthorityService");

    let result = service
        .call(&params)
        .await;

    match result {
        Ok(authority) => {
            info!(
                message = "successfully deleted authority",
                authority = ?authority,
            );

            Response::success().payload(DeleteAuthorityRes { authority })
        },
        Err(err) => {
            info!(
                message = "failed to delete authority",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}

