use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::totp::validate::{ValidateTOTP, ValidateTOTPService};
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::{provider::Provider, response::Response};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTOTPReq {
    pub code: String,
    pub client_key: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTOTPRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}

#[tracing::instrument(name = "validate_totp_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<ValidateTOTPReq>,
) -> impl IntoResponse {
    match parse_and_validate(
        "oxidauth:totp_code:validate",
        &permissions,
    ) {
        Ok(true) => info!(
            "{:?} has {}",
            jwt.sub, "oxidauth:totp_code:validate"
        ),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {}",
                jwt.sub, "oxidauth:totp_code:validate"
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::fail().error(err.to_string()),
    }

    let service = provider.fetch::<ValidateTOTPService>();

    let user_id = match jwt.sub {
        Some(user_id) => user_id,
        None => return Response::unauthorized(),
    };

    let validation_params = ValidateTOTP {
        user_id,
        code: params.code,
        client_key: params.client_key,
    };

    let result = service
        .call(&validation_params)
        .await;

    match result {
        Ok(response) => {
            info!(
                message = "successfully authenticated with 2fa",
                response = ?response,
            );

            Response::success().payload(ValidateTOTPRes {
                jwt: response.jwt,
                refresh_token: response.refresh_token,
            })
        },
        Err(err) => {
            info!(
                message = "failed to authenticate",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
