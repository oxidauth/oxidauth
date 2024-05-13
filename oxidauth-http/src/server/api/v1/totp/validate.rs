use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::service::ExtractPermissions;
use oxidauth_kernel::totp::validate::ValidateTOTPService;
use oxidauth_kernel::{error::IntoOxidAuthError, totp::validate::ValidateTOTP};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::middleware::permission_extractor::ExtractJwt;
use crate::{provider::Provider, response::Response};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTOTPRes {
    code_validation: bool,
}

#[tracing::instrument(name = "validate_totp_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<ValidateTOTPReq>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
) -> impl IntoResponse {
    match parse_and_validate(
        "users.TOTP_code",
        &permissions,
    ) {
        Ok(true) => info!(
            "{:?} has {}",
            jwt.sub, "users.TOTP_code"
        ),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {}",
                jwt.sub, "users.TOTP_code"
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::fail().error(err.to_string()),
    }

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
