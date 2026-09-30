use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    public_keys::delete_public_key::{DeletePublicKeyReq, DeletePublicKeyRes},
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    public_keys::delete_public_key::DeletePublicKeyService,
};
use tracing::info;

use crate::provider::Provider;

#[tracing::instrument(name = "delete_public_key_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<DeletePublicKeyReq>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<DeletePublicKeyService>();

    info!("provided DeletePublicKeyService");

    let result = service
        .delete_public_key(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
