use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    public_keys::list_all_public_keys::{ListAllPublicKeysReq, ListAllPublicKeysRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, public_keys::list_all_public_keys::*};
use tracing::info;

use crate::provider::Provider;

#[tracing::instrument(name = "list_all_public_keys_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<ListAllPublicKeysReq>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<ListAllPublicKeysService>();

    info!("provided ListAllPublicKeysService");

    let result = service
        .list_all_public_keys(&params)
        .await;

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

            Response::bad_request().error(err.into_error())
        },
    }
}
