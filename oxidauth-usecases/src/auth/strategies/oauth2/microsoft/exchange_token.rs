use reqwest::header::CONTENT_TYPE;
use tracing::error;

use crate::auth::strategies::oauth2::AuthorityParams;
use oxidauth_kernel::error::BoxedError;

use super::{MicrosoftExchangeTokenReq, MicrosoftExchangeTokenRes};

#[tracing::instrument(name = "microsoft oauth exchange token")]
pub async fn exchange_microsoft_token(
    code: &String,
    params: &AuthorityParams,
) -> Result<String, BoxedError> {
    let json = MicrosoftExchangeTokenReq {
        code,
        scope: &params.scopes,
        client_id: &params.oauth2_id,
        client_secret: &params.oauth2_secret,
        redirect_uri: &params.redirect_uri.as_ref(),
        grant_type: "authorization_code",
    };

    let exchange_response = reqwest::Client::new()
        .post(params.exchange_url.clone())
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&json)
        .send()
        .await
        .map_err(|err| {
            error!("ERROR IN EXCHANGE AZURE TOKEN {}", err);

            err
        })?;

    let response_text = exchange_response
        .text()
        .await?;

    let exchange =
        serde_json::from_str::<MicrosoftExchangeTokenRes>(&response_text).map_err(|err| {
            error!(
                "ERROR IN EXCHANGE AZURE TOKEN JSON {}: {:?}",
                err, response_text
            );
            err
        })?;

    Ok(exchange.access_token)
}
