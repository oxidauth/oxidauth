use reqwest::header::AUTHORIZATION;
use tracing::error;

use crate::auth::strategies::oauth2::AuthorityParams;
use oxidauth_kernel::{auth::authenticate_or_register::OAuth2Profile, error::BoxedError};

use super::MicrosoftProfile;

pub async fn retrieve_microsoft_profile(
    access_token: String,
    authority_params: &AuthorityParams,
) -> Result<OAuth2Profile, BoxedError> {
    let bearer_token = format!("Bearer {}", &access_token);

    let profile_response = reqwest::Client::new()
        .get(
            authority_params
                .profile_url
                .clone(),
        )
        .header(AUTHORIZATION, bearer_token)
        .send()
        .await
        .map_err(|err| {
            error!("ERROR IN RETRIEVE AZURE PROFILE TEXT {}", err);
            err
        })?;

    let response_text = profile_response
        .text()
        .await?;

    let profile = serde_json::from_str::<MicrosoftProfile>(&response_text).map_err(|err| {
        error!(
            "ERROR IN RETRIEVE AZURE PROFILE JSON {}: {:?}",
            err, response_text
        );
        err
    })?;

    let profile = Ok(OAuth2Profile {
        email: profile.mail,
        given_name: profile.given_name,
        family_name: profile.surname,
    });

    profile
}
