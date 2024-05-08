use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::totp::validate::ValidateTOTPService;
use oxidauth_kernel::{error::IntoOxidAuthError, totp::validate::ValidateTOTP};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{provider::Provider, response::Response};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTOTPReq {
    pub user_id: Uuid,
    pub code: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTOTPRes {
    code_validation: bool,
}

#[tracing::instrument(name = "validate_totp_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<ValidateTOTPReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<ValidateTOTPService>();

    let validation_params = ValidateTOTP {
        user_id: params.user_id,
        code: params.code,
    };

    let result = service
        .call(&validation_params)
        .await;

    match result {
        Ok(response) => Response::success().payload(ValidateTOTPRes {
            code_validation: response.code_validation,
        }),
        Err(err) => Response::fail().error(err.into_error()),
    }
}
