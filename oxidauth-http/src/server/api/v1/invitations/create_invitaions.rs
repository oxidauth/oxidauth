use axum::{extract::State, response::IntoResponse, Json};
pub use oxidauth_kernel::invitations::create_invitation::CreateInvitationParams;
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    invitations::create_invitation::{
        CreateInvitationResponse, CreateInvitationService,
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

pub const PERMISSION: &str = "oxidauth:invitations:create";

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvitationReq {
    pub invitation: CreateInvitationParams,
}

pub type CreateInvitationRes = CreateInvitationResponse;

#[tracing::instrument(name = "create_invitation_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<CreateInvitationReq>,
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

    let service = provider.fetch::<CreateInvitationService>();

    info!("provided CreateInvitationService");

    let result = service
        .call(&params.invitation)
        .await;

    match result {
        Ok(invitation) => {
            info!(
                message = "successfully created invitation",
                invitation = ?invitation,
            );

            Response::success().payload(invitation)
        },
        Err(err) => {
            info!(
                message = "failed to create invitation",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
