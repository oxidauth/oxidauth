use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    settings::fetch_setting::{FetchSettingReq, FetchSettingRes},
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    settings::fetch_setting::{FetchSettingParams, FetchSettingService},
};
use tracing::info;

use crate::provider::Provider;

#[tracing::instrument(name = "fetch_setting_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FetchSettingReq>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<FetchSettingService>();

    info!("provided FetchSettingService");

    let params = FetchSettingParams { key: params.key };

    let result = service
        .fetch_setting(&params)
        .await;

    match result {
        Ok(setting) => {
            info!(
                message = "successfully fetched setting",
                setting_key = setting.key,
            );

            Response::success().payload(FetchSettingRes { setting })
        },
        Err(err) => {
            info!(
                message = "failed to fetch setting",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}
