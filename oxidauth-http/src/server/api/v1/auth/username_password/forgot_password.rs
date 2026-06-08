use axum::{Json, extract::State, response::Response};
use tracing::{error, info};

use oxidauth_kernel::{
    JsonValue,
    auth::username_password::forgot_password::{
        ForgotPasswordInfo, ForgotPasswordParams, ForgotPasswordResponse, ForgotPasswordService,
    },
};

use crate::provider::Provider;

#[tracing::instrument(name = "username_password_forgot_password_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<ForgotPasswordParams>,
) -> Response {
    let service = provider.fetch::<ForgotPasswordService>();

    let result = service
        .call(&ForgotPasswordInfo { id: params.id })
        .await;

    match result {
        Ok(res) => Response::success().payload(ForgotPasswordResponse { code: res.code }),
        Err(err) => Response::fail().error(err.into_error()),
    }
}
