use axum::{extract::State, response::IntoResponse};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::totp::generate::{
    GenerateTOTP, GenerateTOTPService, TOTPCode,
};
use oxidauth_permission::parse_and_validate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::middleware::permission_extractor::{
    ExtractEntitlements, ExtractJwt,
};
use crate::{provider::Provider, response::Response};

pub type TOTPGenerateRes = TOTPCode;

#[derive(Debug, Serialize, Deserialize)]
pub struct TOTPGenerateReq;

#[tracing::instrument(name = "generate_totp_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
) -> impl IntoResponse {
    match parse_and_validate(
        "oxidauth:totp_code:generate",
        &permissions,
    ) {
        Ok(true) => info!(
            "{:?} has {}",
            jwt.sub, "oxidauth:totp_code:generate"
        ),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {}",
                jwt.sub, "oxidauth:totp_code:generate"
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::fail().error(err.to_string()),
    }

    let service = provider.fetch::<GenerateTOTPService>();

    let user_id = match jwt.sub {
        Some(user_id) => user_id,
        None => return Response::unauthorized(),
    };

    let result = service
        .call(&GenerateTOTP { user_id })
        .await;

    match result {
        Ok(response) => {
            info!(
                message = "successfully authenticated with 2fa",
                response = ?response,
            );

            Response::success().payload(TOTPGenerateRes {
                code: response.code,
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
