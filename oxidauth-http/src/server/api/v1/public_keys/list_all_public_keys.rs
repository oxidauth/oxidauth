use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::public_keys::list_all_public_keys::*;
use serde::Serialize;
use tracing::info;

use crate::provider::Provider;
use crate::response::Response;

pub type ListAllPublicKeysReq = ListAllPublicKeys;

#[derive(Debug, Serialize)]
pub struct ListAllPublicKeysRes {
    pub public_keys: Vec<PublicKey>,
}

#[tracing::instrument(name = "list_all_public_keys_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListAllPublicKeysReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ListAllPublicKeysService>();

    info!("provided ListAllPublicKeysService");

    let result = service.call(&params).await;

    match result {
        Ok(public_keys) => {
            info!(
                message = "successfully list public_keys",
                public_keys = ?public_keys,
            );

            Response::success().payload(ListAllPublicKeysRes { public_keys })
        },
        Err(err) => {
            info!(
                message = "failed to list public_keys",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
