use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::role_role_grants::list_role_role_grants_by_parent_id::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type ListRoleRoleGrantsByParentIdReq = ListRoleRoleGrantsByParentId;

#[derive(Debug, Serialize)]
pub struct ListRoleRoleGrantsByParentIdRes {
    pub roles: Vec<RoleRoleGrantDetail>,
}

#[tracing::instrument(
    name = "list_role_role_grants_by_parent_id_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListRoleRoleGrantsByParentIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListRoleRoleGrantsByParentIdService>();

    info!("provided ListRoleRoleGrantsByParentIdService");

    let result = service.call(&params).await;

    match result {
        Ok(roles) => {
            info!(
                message = "successfully listed role role grants by parent_id",
                roles = ?roles,
            );

            Response::success()
                .payload(ListRoleRoleGrantsByParentIdRes { roles })
        },
        Err(err) => {
            info!(
                message = "failed to list role role grants by parent_id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
