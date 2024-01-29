use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::settings::{
    fetch_setting::{FetchSettingParams, FetchSettingService},
    Setting,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{provider::Provider, response::Response};

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchSettingReq {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchSettingRes {
    pub setting: Setting,
}

#[tracing::instrument(name = "fetch_setting_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(params): Path<FetchSettingReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<FetchSettingService>();

    info!("provided FetchSettingService");

    let params = FetchSettingParams { key: params.key };

    let result = service.call(&params).await;

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

            Response::fail().error(err.into_error())
        },
    }
}
