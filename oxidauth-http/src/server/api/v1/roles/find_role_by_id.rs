use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::roles::find_role_by_id::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type FindRoleByIdReq = FindRoleById;

#[derive(Debug, Serialize)]
pub struct FindRoleByIdRes {
    pub role: Role,
}

#[tracing::instrument(name = "find_role_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FindRoleByIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<FindRoleByIdService>();

    info!("provided FindRoleByIdService");

    let result = service.call(&params).await;

    match result {
        Ok(role) => {
            info!(
                message = "successfully found role by id",
                role = ?role,
            );

            Response::success().payload(FindRoleByIdRes { role })
        },
        Err(err) => {
            info!(
                message = "failed to find role by id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
