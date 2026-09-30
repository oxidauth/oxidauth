use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    authorities::find_authority_by_strategy::{
        FindAuthorityByStrategyReq,
        FindAuthorityByStrategyRes,
    },
};
use oxidauth_kernel::{authorities::find_authority_by_strategy::*, error::IntoOxidAuthError};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "find_authority_by_strategy_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindAuthorityByStrategyReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<FindAuthorityByStrategyService>();
    info!("provided FindAuthorityByStrategyService");
    let result = service
        .find_authority_by_strategy(&params)
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

            Response::bad_request().error(err.into_error())
        },
    }
}
