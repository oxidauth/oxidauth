use uuid::Uuid;
use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::role_role_grants::list_role_role_grants_by_parent_id::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct ListRoleRoleGrantsByParentIdReq {
    pub parent_id: Uuid,
}

impl From<ListRoleRoleGrantsByParentIdReq> for ListRoleRoleGrantsByParentId {
    fn from(value: ListRoleRoleGrantsByParentIdReq) -> Self {
        Self {
            parent_id: value.parent_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateRoleRoleGrantRes {
    pub roles: Vec<RoleRoleGrantDetail>,
}

#[tracing::instrument(name = "list_role_role_grants_by_parent_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListRoleRoleGrantsByParentIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListRoleRoleGrantsByParentIdService>();

    info!("provided ListRoleRoleGrantsByParentIdService");

    let result = service
        .call(&params.into())
        .await;

    match result {
        Ok(roles) => {
            info!(
                message = "successfully listed role role grants by parent_id",
                roles = ?roles,
            );

            Response::success().payload(CreateRoleRoleGrantRes { roles })
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

