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
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{provider::Provider, response::Response};

#[derive(Debug, Deserialize)]
pub struct DeletePublicKeyReq {
    pub public_key_id: Uuid,
}

#[derive(Debug, Serialize)]
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

    let result = service
        .call(&params.into())
        .await;

    match result {
        Ok(public_key) => {
            info!(
                message = "successfully found public_key by id",
                public_key = ?public_key,
            );

            Response::success().payload(DeletePublicKeyRes { public_key })
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

impl From<DeletePublicKeyReq> for DeletePublicKey {
    fn from(value: DeletePublicKeyReq) -> Self {
        Self {
            public_key_id: value.public_key_id,
        }
    }
}
