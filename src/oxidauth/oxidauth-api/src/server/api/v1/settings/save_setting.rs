use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{
    Response,
    settings::save_setting::{SaveSettingReq, SaveSettingRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, settings::save_setting::SaveSettingService};
use tracing::info;

use crate::provider::Provider;

#[tracing::instrument(name = "save_setting_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<SaveSettingReq>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<SaveSettingService>();

    info!("provided SaveSettingService");

    let result = service
        .save_setting(&params.setting)
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

            Response::bad_request().error(err.into_error())
        },
    }
}
