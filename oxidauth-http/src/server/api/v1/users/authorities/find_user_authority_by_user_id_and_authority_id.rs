use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::{error::IntoOxidAuthError, user_authorities::UserAuthorityWithAuthority};
use oxidauth_kernel::user_authorities::find_user_authority_by_user_id_and_authority_id::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::provider::Provider;
use crate::response::Response;

#[derive(Debug, Deserialize)]
pub struct FindUserAuthorityByUserIdAndAuthorityIdReq {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct FindUserAuthorityByUserIdAndAuthorityIdRes {
    pub user_authority: UserAuthorityWithAuthority,
}

#[tracing::instrument(
    name = "find_user_authority_by_user_id_and_authority_id_handler",
    skip(provider)
)]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FindUserAuthorityByUserIdAndAuthorityIdReq>,
) -> impl IntoResponse {
    let service =
        provider.fetch::<FindUserAuthorityByUserIdAndAuthorityIdService>();

    info!("provided FindUserAuthorityByUserIdAndAuthorityIdService");

    let result = service
        .call(
            &FindUserAuthorityByUserIdAndAuthorityId {
                user_id: params.user_id,
                authority_id: params.authority_id,
            },
        )
        .await;

    match result {
        Ok(user_authority) => {
            info!(
                message = "successfully found user_authority by user_id and authority_id",
                user_authority = ?user_authority,
            );

            Response::success().payload(
                FindUserAuthorityByUserIdAndAuthorityIdRes { user_authority },
            )
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
