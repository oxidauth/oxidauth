use oxidauth_kernel::{roles::Role, permissions::Permission};
use uuid::Uuid;
use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::role_permission_grants::create_role_permission_grant::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct CreateRolePermissionGrantReq {
    pub role_id: Uuid,
    pub permission: String,
}

impl From<CreateRolePermissionGrantReq> for CreateRolePermissionGrant {
    fn from(value: CreateRolePermissionGrantReq) -> Self {
        Self {
            role_id: value.role_id,
            permission: value.permission,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateRolePermissionGrantRes {
    pub permission: Permission,
    pub grant: RolePermissionGrant,
}

#[tracing::instrument(name = "create_role_permission_grant_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<CreateRolePermissionGrantReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<CreateRolePermissionGrantService>();

    info!("provided CreateRolePermissionGrantService");

    let result = service
        .call(&params.into())
        .await;

    match result {
        Ok(res) => {
            info!(
                message = "successfully created role permission grant",
                res = ?res,
            );

            Response::success().payload(CreateRolePermissionGrantRes { permission: res.permission, grant: res.grant })
        },
        Err(err) => {
            info!(
                message = "failed to create role permission grant",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
