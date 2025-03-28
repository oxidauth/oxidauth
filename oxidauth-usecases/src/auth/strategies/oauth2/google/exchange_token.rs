use reqwest::header::CONTENT_TYPE;

use crate::auth::strategies::oauth2::AuthorityParams;
use oxidauth_kernel::error::BoxedError;

use super::{GoogleExchangeTokenReq, GoogleExchangeTokenRes};

#[tracing::instrument(name = "google oauth exchange token")]
pub async fn exchange_token(code: String, params: &AuthorityParams) -> Result<String, BoxedError> {
    let json = GoogleExchangeTokenReq {
        code,
        client_id: params.oauth2_id.clone(),
        client_secret: params.oauth2_secret.clone(),
        redirect_uri: params
            .redirect_uri
            .to_string(),
        grant_type: "authorization_code".to_string(),
    };

    let exchange: GoogleExchangeTokenRes = reqwest::Client::new()
        .post(params.exchange_url.clone())
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&json)
        .send()
        .await
        .map_err(|err| err.to_string())?
        .json()
        .await
        .map_err(|err| err.to_string())?;

    Ok(exchange.access_token)
}
