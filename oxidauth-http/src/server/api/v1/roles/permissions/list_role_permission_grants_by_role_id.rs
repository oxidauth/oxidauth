use uuid::Uuid;
use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct ListRolePermissionGrantsByRoleIdReq {
    pub role_id: Uuid,
}

impl From<ListRolePermissionGrantsByRoleIdReq> for ListRolePermissionGrantsByRoleId {
    fn from(value: ListRolePermissionGrantsByRoleIdReq) -> Self {
        Self {
            role_id: value.role_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListRolePermissionGrantsByRoleIdRes {
    pub permissions: Vec<RolePermission>,
}

#[tracing::instrument(name = "list_role_permission_grants_by_role_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListRolePermissionGrantsByRoleIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListRolePermissionGrantsByRoleIdService>();

    info!("provided ListRolePermissionGrantsByRoleIdService");

    let result = service
        .call(&params.into())
        .await;

    match result {
        Ok(permissions) => {
            info!(
                message = "successfully listed role permission grants by role_id",
                permissions = ?permissions,
            );

            Response::success().payload(ListRolePermissionGrantsByRoleIdRes { permissions })
        },
        Err(err) => {
            info!(
                message = "failed to list role permission grants by role_id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}

