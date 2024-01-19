use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::authorities::find_authority_by_strategy::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type FindAuthorityByStrategyReq = FindAuthorityByStrategy;

#[derive(Debug, Serialize)]
pub struct FindAuthorityByStrategyRes {
    pub authority: Authority,
}

#[tracing::instrument(name = "find_authority_by_strategy_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FindAuthorityByStrategyReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<FindAuthorityByStrategyService>();

    info!("provided FindAuthorityByStrategyService");

    let result = service
        .call(&params)
        .await;

    match result {
        Ok(authority) => {
            info!(
                message = "successfully found authority by strategy",
                authority = ?authority,
            );

            Response::success().payload(FindAuthorityByStrategyRes { authority })
        },
        Err(err) => {
            info!(
                message = "failed to find authority by strategy",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}

