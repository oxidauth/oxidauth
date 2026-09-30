use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    authorities::update_authority::{
        UpdateAuthorityPathReq,
        UpdateAuthorityReq,
        UpdateAuthorityRes,
    },
};
use oxidauth_kernel::{authorities::update_authority::*, error::IntoOxidAuthError};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "update_authority_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(path): Path<UpdateAuthorityPathReq>,
    Json(mut params): Json<UpdateAuthorityReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<UpdateAuthorityService>();

    info!("provided UpdateAuthorityService");

    params.authority.id = Some(path.authority_id);

    let result = service
        .update_authority(&mut params.authority)
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

            Response::bad_request().error(err.into_error())
        },
    }
}
