use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::roles::list_all_roles::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

type ListAllRolesReq = ListAllRoles;

#[derive(Debug, Serialize)]
pub struct ListAllRolesRes {
    pub roles: Vec<Role>,
}

#[tracing::instrument(name = "list_all_roles_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListAllRolesReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListAllRolesService>();

    info!("provided ListAllRolesService");

    let result = service
        .call(&params.into())
        .await;

    match result {
        Ok(roles) => {
            info!(
                message = "successfully listed roles",
                roles = ?roles,
            );

            Response::success().payload(ListAllRolesRes { roles })
        },
        Err(err) => {
            info!(
                message = "failed to list roles",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
