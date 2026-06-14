use axum::{Json, extract::State, response::IntoResponse};

use oxidauth_kernel::{
    auth::username_password::forgot_password::{
        ForgotPasswordParams, ForgotPasswordResponse, ForgotPasswordService,
    },
    error::IntoOxidAuthError,
};

use crate::{provider::Provider, response::Response};

#[tracing::instrument(name = "username_password_forgot_password_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<ForgotPasswordParams>,
) -> impl IntoResponse {
    let service = provider.fetch::<ForgotPasswordService>();

    let result = service.call(&params).await;

    match result {
        Ok(res) => Response::success().payload(ForgotPasswordResponse { code: res.code }),
        Err(err) => Response::fail().error(err.into_error()),
    }
}
