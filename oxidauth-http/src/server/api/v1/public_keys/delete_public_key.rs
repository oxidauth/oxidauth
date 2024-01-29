use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    public_keys::{
        delete_public_key::{DeletePublicKey, DeletePublicKeyService},
        PublicKey,
    },
};
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::{provider::Provider, response::Response};

pub type DeletePublicKeyReq = DeletePublicKey;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePublicKeyRes {
    pub public_key: PublicKey,
}

#[tracing::instrument(name = "delete_public_key_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeletePublicKeyReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<DeletePublicKeyService>();

    info!("provided DeletePublicKeyService");

    let result = service.call(&params).await;

    match result {
        Ok(public_key) => {
            info!(
                message = "successfully deleted public_key",
                public_key = ?public_key,
            );

            Response::success().payload(DeletePublicKeyRes { public_key })
        },
        Err(err) => {
            info!(
                message = "failed to delete public_key",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
