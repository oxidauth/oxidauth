use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    authorities::list_all_authorities::{ListAllAuthoritiesReq, ListAllAuthoritiesRes},
};
use oxidauth_kernel::{authorities::list_all_authorities::*, error::IntoOxidAuthError};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "list_all_authorities_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListAllAuthoritiesReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<ListAllAuthoritiesService>();

    info!("provided ListAllAuthoritiesService");

    let result = service
        .list_all_authorities(&params)
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

            Response::bad_request().error(err.into_error())
        },
    }
}
