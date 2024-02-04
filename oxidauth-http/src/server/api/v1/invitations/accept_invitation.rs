use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
pub use oxidauth_kernel::invitations::accept_invitation::*;
use oxidauth_kernel::{
    auth::register::RegisterParams, error::IntoOxidAuthError,
    provider::Provider, users::User,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::response::Response;

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptInvitationPathReq {
    pub invitation_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptInvitationBodyReq {
    pub user: AcceptInvitationUserParams,
    pub user_authority: RegisterParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptInvitationRes {
    pub user: User,
}

#[tracing::instrument(name = "accept_invitation_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(path): Path<AcceptInvitationPathReq>,
    Json(body): Json<AcceptInvitationBodyReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<AcceptInvitationService>();

    info!("provided AcceptInvitationService");

    let result = service
        .call(&AcceptInvitationParams {
            invitation_id: path.invitation_id,
            user: body.user,
            user_authority: body.user_authority,
        })
        .await;

    match result {
        Ok(user) => {
            info!(
                message = "successfully accepted invitation",
                user = ?user,
            );

            Response::success().payload(AcceptInvitationRes { user })
        },
        Err(err) => {
            info!(
                message = "failed to accept invitation",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
