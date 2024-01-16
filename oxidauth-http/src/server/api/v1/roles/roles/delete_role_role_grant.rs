use oxidauth_kernel::roles::Role;
use uuid::Uuid;
use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::role_role_grants::delete_role_role_grant::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct DeleteRoleRoleGrantReq {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}

impl From<DeleteRoleRoleGrantReq> for DeleteRoleRoleGrant {
    fn from(value: DeleteRoleRoleGrantReq) -> Self {
        Self {
            parent_id: value.parent_id,
            child_id: value.child_id,
        }
    }
}

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

    let result = service
        .call(&params.into())
        .await;

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

