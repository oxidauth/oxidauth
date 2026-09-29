use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    invitations::{
        delete_invitation::{DeleteInvitationParams, DeleteInvitationService},
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

pub const PERMISSION: &str = "oxidauth:invitations:delete";

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteInvitationReq {
    pub invitation: DeleteInvitationParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteInvitationRes {
    pub invitation: Invitation,
}

#[tracing::instrument(name = "delete_invitation_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<DeleteInvitationReq>,
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

    let service = provider.fetch::<DeleteInvitationService>();

    info!("provided DeleteInvitationService");

    let result = service
        .call(&params.invitation)
        .await;

    match result {
        Ok(invitation) => {
            info!(
                message = "successfully deleted invitation",
                invitation = ?invitation,
            );

            Response::success().payload(DeleteInvitationRes { invitation })
        },
        Err(err) => {
            info!(
                message = "failed to delete invitation",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
