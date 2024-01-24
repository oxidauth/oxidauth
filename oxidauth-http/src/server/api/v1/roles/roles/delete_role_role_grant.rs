use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::role_role_grants::delete_role_role_grant::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type DeleteRoleRoleGrantReq = DeleteRoleRoleGrant;

#[derive(Debug, Serialize)]
pub struct DeleteRoleRoleGrantRes {
    pub grant: RoleRoleGrant,
}

#[tracing::instrument(name = "delete_role_role_grant_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeleteRoleRoleGrantReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeleteRoleRoleGrantService>();

    info!("provided DeleteRoleRoleGrantService");

    let result = service.call(&params).await;

    match result {
        Ok(grant) => {
            info!(
                message = "successfully deleted role role grant",
                grant = ?grant,
            );

            Response::success().payload(DeleteRoleRoleGrantRes { grant })
        },
        Err(err) => {
            info!(
                message = "failed to delete role role grant",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
