use oxidauth_kernel::{roles::Role, permissions::Permission};
use uuid::Uuid;
use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::role_permission_grants::delete_role_permission_grant::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct DeleteRolePermissionGrantReq {
    pub role_id: Uuid,
    pub permission: String,
}

impl From<DeleteRolePermissionGrantReq> for DeleteRolePermissionGrant {
    fn from(value: DeleteRolePermissionGrantReq) -> Self {
        Self {
            role_id: value.role_id,
            permission: value.permission,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DeleteRolePermissionGrantRes {
    pub permission: Permission,
    pub grant: RolePermissionGrant,
}

#[tracing::instrument(name = "delete_role_permission_grant_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeleteRolePermissionGrantReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeleteRolePermissionGrantService>();

    info!("provided DeleteRolePermissionGrantService");

    let result = service
        .call(&params.into())
        .await;

    match result {
        Ok(res) => {
            info!(
                message = "successfully deleted role permission grant",
                res = ?res,
            );

            Response::success().payload(DeleteRolePermissionGrantRes { permission: res.permission, grant: res.grant })
        },
        Err(err) => {
            info!(
                message = "failed to delete role permission grant",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}

