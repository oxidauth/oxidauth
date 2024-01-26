use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::roles::delete_role::*;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type DeleteRoleReq = DeleteRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRoleRes {
    pub role: Role,
}

#[tracing::instrument(name = "delete_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeleteRoleReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeleteRoleService>();

    info!("provided DeleteRoleService");

    let result = service.call(&params).await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully deleted role",
                role = ?role,
            );

            Response::success().payload(DeleteRoleRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to delete role",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
