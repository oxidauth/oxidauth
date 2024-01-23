use axum::{extract::State, response::IntoResponse};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    public_keys::{
        create_public_key::{CreatePublicKey, CreatePublicKeyService},
        PublicKey,
    },
};
use serde::Serialize;
use tracing::info;

use crate::{provider::Provider, response::Response};

#[derive(Debug, Serialize)]
pub struct CreatePublicKeyRes {
    pub public_key: PublicKey,
}

#[tracing::instrument(name = "create_public_key_handler", skip(provider))]
pub async fn handle(State(provider): State<Provider>) -> impl IntoResponse {
    let service = provider.fetch::<CreatePublicKeyService>();

    info!("provided CreatePublicKeyService");

    let result = service
        .call(&CreatePublicKey())
        .await;

    match result {
        Ok(public_key) => {
            info!(
                message = "successfully created public key",
                public_key = ?public_key,
            );

            Response::success().payload(CreatePublicKeyRes { public_key })
        },
        Err(err) => {
            info!(
                message = "failed to create public_key",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
