use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::roles::create_role::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct CreateRoleReq {
    pub role: CreateRole,
}

#[derive(Debug, Serialize)]
pub struct CreateRoleRes {
    pub role: Role,
}

#[tracing::instrument(name = "create_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<CreateRoleReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<CreateRoleService>();

    info!("provided CreateRoleService");

    let result = service
        .call(&params.role)
        .await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully created role",
                role = ?role,
            );

            Response::success().payload(CreateRoleRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to create role",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
