use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    users::roles::list_user_roles_by_user_id::{
        ListUserRoleGrantsByUserIdReq,
        ListUserRoleGrantsByUserIdRes,
    },
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    user_role_grants::list_user_role_grants_by_user_id::*,
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use super::PERMISSION;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "list_user_role_grants_by_user_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<ListUserRoleGrantsByUserIdReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<ListUserRoleGrantsByUserIdService>();

    info!("provided ListUserRoleGrantsByUserIdService");

    let result = service
        .list_user_role_grants_by_user_id(&params)
        .await;

    match result {
        Ok(user_role_grants) => {
            info!(
                message = "successfully listing user role grants by user_id",
                user_role_grants = ?user_role_grants,
            );

            Response::success().payload(ListUserRoleGrantsByUserIdRes { user_role_grants })
        },
        Err(err) => {
            info!(
                message = "failed to list user role grants by user_id",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
