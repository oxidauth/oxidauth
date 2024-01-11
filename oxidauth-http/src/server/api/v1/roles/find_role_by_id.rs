use uuid::Uuid;
use axum::{extract::{Path, State}, response::IntoResponse};
use oxidauth_kernel::roles::find_role_by_id::*;
use oxidauth_kernel::error::IntoOxidAuthError;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct FindRoleByIdReq {
    pub role_id: Uuid,
}


#[allow(clippy::from_over_into)]
impl Into<FindRoleById> for FindRoleByIdReq {
    fn into(self) -> FindRoleById {
        FindRoleById {
            role_id: self.role_id,
        }
    }
}

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

    let result = service
        .call(&params.into())
        .await;

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

