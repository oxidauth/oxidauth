use uuid::Uuid;
use axum::{extract::{Path, State}, response::IntoResponse, Json};
use oxidauth_kernel::roles::update_role::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct UpdateRolePathReq {
    pub role_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleReq {
    pub role: UpdateRole,
}

#[derive(Debug, Serialize)]
pub struct UpdateRoleRes {
    pub role: Role,
}

#[tracing::instrument(name = "update_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<UpdateRolePathReq>,
    Json(body): Json<UpdateRoleReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<UpdateRoleService>();

    info!("provided UpdateRoleService");

    let mut updates = body.role;

    updates.role_id = Some(params.role_id);

    let result = service
        .call(&updates)
        .await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully updated role",
                role = ?role,
            );

            Response::success().payload(UpdateRoleRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to update role",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
