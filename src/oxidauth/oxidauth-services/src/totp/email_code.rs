use std::time::Duration;

use boringauth::oath::TOTPBuilder;
use oxidauth_kernel::{
    auth::authenticate::{WebhookReq, WebhookRes},
    error::BoxedError,
    users::User,
};
use reqwest::Client;
use url::Url;

use crate::dev_prelude::epoch;

/// Generate the code for `secret` inside the authority's own window and hand it to
/// the customer webhook: oxidauth never sends mail itself, the application does.
///
/// Both legs that open a TOTP session arrive here — the first leg of a login
/// (`auth::authenticate`) and a registration on a TOTP authority
/// (`auth::register`) — so the code, the window it lives in, and the failure the
/// caller sees cannot drift apart between them. A code that never arrived is a hard
/// error rather than a quiet pass: the jwt minted alongside it is only good for
/// `oxidauth:totp_code:validate`, so a session that cannot get its code dies at the
/// gate instead of degrading into a password-only one.
pub async fn email_code(
    webhook: Url,
    webhook_key: String,
    totp_ttl: Duration,
    secret: &str,
    user: &User,
    user_identifier: &str,
) -> Result<(), BoxedError> {
    let now = epoch()?;

    let code = TOTPBuilder::new()
        .ascii_key(secret)
        .period(totp_ttl.as_secs() as u32)
        .timestamp(now)
        .finalize()
        .map_err(|err| format!("error generating totp: {:?}", err))?
        .generate();

    let name = match (user.first_name.as_deref(), user.last_name.as_deref()) {
        (None, None) => None,
        (None, Some(last)) => Some(last.to_owned()),
        (Some(first), None) => Some(first.to_owned()),
        (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
    };

    let email = user
        .email
        .as_deref()
        .ok_or("unable to send totp code - missing email")?;

    let webhook_params = WebhookReq {
        webhook_key,
        name,
        email: email.to_owned(),
        code,
    };

    let client = Client::new();
    let webhook_res: WebhookRes = client
        .post(webhook)
        .json(&webhook_params)
        .send()
        .await?
        .json()
        .await?;

    if !webhook_res.success {
        return Err(format!("unable to send totp code to: {}", user_identifier).into());
    }

    Ok(())
}
