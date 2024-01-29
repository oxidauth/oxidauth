use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::public_keys::find_public_key_by_id::*;
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type FindPublicKeyByIdReq = FindPublicKeyById;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPublicKeyByIdRes {
    pub public_key: PublicKey,
}

#[tracing::instrument(name = "find_public_key_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FindPublicKeyByIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<FindPublicKeyByIdService>();

    info!("provided FindPublicKeyByIdService");

    let result = service.call(&params).await;

    match result {
        Ok(public_key) => {
            info!(
                message = "successfully found public_key by id",
                public_key = ?public_key,
            );

            Response::success().payload(FindPublicKeyByIdRes { public_key })
        },
        Err(err) => {
            info!(
                message = "failed to find public_key by id",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
