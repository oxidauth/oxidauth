use axum::{Json, extract::State, response::IntoResponse};

use oxidauth_kernel::auth::username_password::update_password::{
    UpdatePasswordParams, UpdatePasswordResponse, UpdatePasswordService,
};

use crate::{provider::Provider, response::Response};

#[axum::debug_handler]
#[tracing::instrument(name = "username_password_update_password_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<UpdatePasswordParams>,
) -> impl IntoResponse {
    let service = provider.fetch::<UpdatePasswordService>();

    let result = service.call(&params).await;

    match result {
        Ok(_) => Response::success().payload(UpdatePasswordResponse { success: true }),
        Err(_) => Response::success().payload(UpdatePasswordResponse { success: false }),
    }
}
