use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::authorities::list_all_authorities::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type ListAllAuthoritiesReq = ListAllAuthorities;

#[derive(Debug, Serialize)]
pub struct ListAllAuthoritiesRes {
    pub authorities: Vec<Authority>,
}

#[tracing::instrument(name = "list_all_authorities_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListAllAuthoritiesReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListAllAuthoritiesService>();

    info!("provided ListAllAuthoritiesService");

    let result = service
        .call(&params)
        .await;

    match result {
        Ok(authorities) => {
            info!(
                message = "successfully listed all authorities",
                authorities = ?authorities,
            );

            Response::success().payload(ListAllAuthoritiesRes { authorities })
        },
        Err(err) => {
            info!(
                message = "failed to list all authorities",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
