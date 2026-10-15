use oxidauth_kernel::error::BoxedError;
use reqwest::header::CONTENT_TYPE;

use super::{AppleAuthError, AppleExchangeTokenReq, AppleExchangeTokenRes, build_client_secret};
use crate::auth::strategies::oauth2::AuthorityParams;

/// Exchanges the authorization code at Apple's token endpoint and returns the
/// **id_token** (Apple's identity carrier — the access token has no userinfo
/// endpoint to spend it at, OXA-000072). The client secret posted along is a
/// freshly signed ES256 JWT, not a stored string.
#[tracing::instrument(name = "apple oauth exchange token")]
pub async fn exchange_apple_token(
    code: String,
    params: &AuthorityParams,
) -> Result<String, BoxedError> {
    let client_secret = build_client_secret(params)?;

    let form = AppleExchangeTokenReq {
        code: &code,
        client_id: &params.oauth2_id,
        client_secret: &client_secret,
        redirect_uri: params.redirect_uri.as_ref(),
        grant_type: "authorization_code",
    };

    let response = reqwest::Client::new()
        .post(params.exchange_url.clone())
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&form)
        .send()
        .await
        .map_err(|_| AppleAuthError::Transport)?;

    let status = response.status();

    if !status.is_success() {
        // capture only the machine-readable OAuth error code; raw IdP bodies
        // stay in the redacted log line, never in the error value
        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or(serde_json::Value::Null);
        let error = body
            .get("error")
            .and_then(|v| v.as_str())
            .map(str::to_owned);

        tracing::warn!(
            status = status.as_u16(),
            error = error
                .as_deref()
                .unwrap_or("unknown"),
            "apple token endpoint rejected the exchange",
        );

        return Err(AppleAuthError::IdpRejected {
            status: status.as_u16(),
            error,
        }
        .into());
    }

    let exchange: AppleExchangeTokenRes = response
        .json()
        .await
        .map_err(|_| AppleAuthError::MissingIdToken)?;

    Ok(exchange.id_token)
}
