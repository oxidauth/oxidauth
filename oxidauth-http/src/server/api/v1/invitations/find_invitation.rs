use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    invitations::{
        find_invitation::{FindInvitationParams, FindInvitationService},
        Invitation,
    },
};
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::provider::Provider;
use crate::response::Response;

pub const PERMISSION: &str = "oxidauth:invitations:read";

pub type FindInvitationReq = FindInvitationParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindInvitationRes {
    pub invitation: Invitation,
}

#[tracing::instrument(name = "find_invitation_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindInvitationReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!(
            "{:?} has {}",
            jwt.sub, PERMISSION
        ),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {}",
                jwt.sub, PERMISSION
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::fail().error(err.to_string()),
    }

    let service = provider.fetch::<FindInvitationService>();

    info!("provided FindInvitationService");

    let result = service.call(&params).await;

    match result {
        Ok(invitation) => {
            info!(
                message = "successfully found invitation",
                invitation = ?invitation,
            );

            Response::success().payload(FindInvitationRes { invitation })
        },
        Err(err) => {
            info!(
                message = "failed to find invitation",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
