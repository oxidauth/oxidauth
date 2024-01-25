use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::{error::IntoOxidAuthError, user_authorities::UserAuthorityWithAuthority};
use oxidauth_kernel::user_authorities::find_user_authority_by_user_id_and_authority_id::*;
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use crate::response::Response;
use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

use super::PERMISSION;

pub type FindUserAuthorityByUserIdAndAuthorityIdReq =
    FindUserAuthorityByUserIdAndAuthorityId;

pub type FindUserAuthorityByUserIdAndAuthorityIdRes =
    UserAuthorityWithAuthority;

#[tracing::instrument(
    name = "find_user_authority_by_user_id_and_authority_id_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindUserAuthorityByUserIdAndAuthorityIdReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!(
            "{:?} has {}",
            jwt.sub, PERMISSION
        ),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {}",
                jwt.sub, PERMISSION
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::fail().error(err.to_string()),
    }

    let service =
        provider.fetch::<FindUserAuthorityByUserIdAndAuthorityIdService>();

    info!("provided FindUserAuthorityByUserIdAndAuthorityIdService");

    let result = service.call(&params).await;

    match result {
        Ok(user_authority) => {
            info!(
                message = "successfully found user_authority by user_id and authority_id",
                user_authority = ?user_authority,
            );

            Response::success().payload(user_authority)
        },
        Err(err) => {
            info!(
                message = "failed to find user_authority by user_id and authority_id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
