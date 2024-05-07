use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::totp_codes::validate::ValidateTOTPService;
use oxidauth_kernel::totp_codes::TOTPValidation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{provider::Provider, response::Response};

// Take in - user id? Or username or something else?
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTOTPReq {
    pub user_id: Uuid,
    pub code: u32,
}

#[tracing::instrument(name = "totp_codes_validate_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<ValidateTOTPReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ValidateTOTPService>();

    let result = service.call(&params).await;

    match result {
        Ok(response) => Response::success().payload(TOTPValidation {
            code_validation: response.jwt,
        }),
        Err(err) => Response::fail().error(err.into_error()),
    }
}
