use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::settings::{
    save_setting::{SaveSettingParams, SaveSettingService},
    Setting,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{provider::Provider, response::Response};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveSettingReq {
    pub setting: SaveSettingParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveSettingRes {
    pub setting: Setting,
}

#[tracing::instrument(name = "save_setting_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<SaveSettingReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<SaveSettingService>();

    info!("provided SaveSettingService");

    let result = service
        .call(&params.setting)
        .await;

    match result {
        Ok(setting) => {
            info!(
                message = "successfully saved setting",
                setting_key = setting.key,
            );

            Response::success().payload(SaveSettingRes { setting })
        },
        Err(err) => {
            info!(
                message = "failed to save setting",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
