use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::role_role_grants::create_role_role_grant::*;
use oxidauth_kernel::{error::IntoOxidAuthError, roles::Role};
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type CreateRoleRoleGrantReq = CreateRoleRoleGrant;

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

    let result = service.call(&params).await;

    match result {
        Ok(res) => {
            info!(
                message = "successfully created role role grant",
                res = ?res,
            );

            Response::success().payload(CreateRoleRoleGrantRes {
                child: res.role,
                grant: res.grant,
            })
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
