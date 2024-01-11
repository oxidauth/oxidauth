use uuid::Uuid;
use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::roles::delete_role::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct DeleteRoleReq {
    pub role_id: Uuid,
}


#[allow(clippy::from_over_into)]
impl Into<DeleteRole> for DeleteRoleReq {
    fn into(self) -> DeleteRole {
        DeleteRole {
            role_id: self.role_id,
        }
    }
}

#[derive(Debug, Serialize)]
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

    let result = service
        .call(&params.into())
        .await;

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


