use oxidauth_kernel::roles::Role;
use uuid::Uuid;
use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::role_role_grants::create_role_role_grant::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct CreateRoleRoleGrantReq {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}

impl From<CreateRoleRoleGrantReq> for CreateRoleRoleGrant {
    fn from(value: CreateRoleRoleGrantReq) -> Self {
        Self {
            parent_id: value.parent_id,
            child_id: value.child_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateRoleRoleGrantRes {
    pub child: Role,
    pub grant: RoleRoleGrant,
}

#[tracing::instrument(name = "create_role_role_grant_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<CreateRoleRoleGrantReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<CreateRoleRoleGrantService>();

    info!("provided CreateRoleRoleGrantService");

    let result = service
        .call(&params.into())
        .await;

    match result {
        Ok(res) => {
            info!(
                message = "successfully created role role grant",
                res = ?res,
            );

            Response::success().payload(CreateRoleRoleGrantRes { child: res.child, grant: res.grant })
        },
        Err(err) => {
            info!(
                message = "failed to create role role grant",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
