use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::user_role_grants::create_user_role_grant::*;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type CreateUserRoleReq = CreateUserRoleGrant;

pub type CreateUserRoleRes = UserRole;

#[tracing::instrument(name = "create_user_role_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<CreateUserRoleReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<CreateUserRoleGrantService>();

    info!("provided CreateUserRoleGrantService");

    let result = service.call(&params).await;

    match result {
        Ok(user_role) => {
            info!(
                message = "successfully created user role",
                user_role = ?user_role,
            );

            Response::success().payload(user_role)
        },
        Err(err) => {
            info!(
                message = "failed to create user role",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
