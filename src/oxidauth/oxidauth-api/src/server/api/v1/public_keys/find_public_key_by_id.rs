use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    public_keys::find_public_key_by_id::{FindPublicKeyByIdReq, FindPublicKeyByIdRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, public_keys::find_public_key_by_id::*};
use tracing::info;

use crate::provider::Provider;

#[tracing::instrument(name = "find_public_key_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FindPublicKeyByIdReq>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<FindPublicKeyByIdService>();

    info!("provided FindPublicKeyByIdService");

    let result = service
        .find_public_key_by_id(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
