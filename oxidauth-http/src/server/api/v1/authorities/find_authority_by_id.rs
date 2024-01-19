use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::authorities::find_authority_by_id::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type FindAuthorityByIdReq = FindAuthorityById;

#[derive(Debug, Serialize)]
pub struct FindAuthorityByIdRes {
    pub authority: Authority,
}

#[tracing::instrument(name = "find_authority_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FindAuthorityByIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<FindAuthorityByIdService>();

    info!("provided FindAuthorityByIdService");

    let result = service
        .call(&params)
        .await;

    match result {
        Ok(authority) => {
            info!(
                message = "successfully found authority by id",
                authority = ?authority,
            );

            Response::success().payload(FindAuthorityByIdRes { authority })
        },
        Err(err) => {
            info!(
                message = "failed to find authority by id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
