use axum::{Json, extract::State, response::Response};
use tracing::{error, info};

use oxidauth_kernel::{
    JsonValue,
    auth::username_password::reset_password::{ResetPasswordParams, ResetPasswordService},
};

use crate::provider::Provider;

#[tracing::instrument(name = "username_password_reset_password_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<ResetPasswordParams>,
) -> Response {
    let service = provider.fetch::<ResetPasswordService>();

    let result = service
        .call(&AuthenticateOrRegisterParams { email })
        .await;

    match result {
        Ok(_) => Response::success(),
        Err(_) => Response::failure(),
    }
}
