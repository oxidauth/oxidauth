use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::error::IntoOxidAuthError;
use oxidauth_kernel::totp::generate::{GenerateTOTP, GenerateTOTPService};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{provider::Provider, response::Response};

// @ALYSSA Take in ... user id? Or username?
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateTOTPReq {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateTOTPRes {
    pub code: u32,
}

#[tracing::instrument(name = "totp_generate_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<GenerateTOTPReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<GenerateTOTPService>();

    let generate_params = GenerateTOTP {
        user_id: params.user_id,
    };

    let result = service
        .call(&generate_params)
        .await;

    match result {
        Ok(response) => Response::success().payload(GenerateTOTPRes {
            code: response.code,
        }),
        Err(err) => Response::fail().error(err.into_error()),
    }
}
