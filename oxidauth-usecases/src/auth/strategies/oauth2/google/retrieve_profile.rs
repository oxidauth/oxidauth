use reqwest::header::AUTHORIZATION;
use tracing::error;

use crate::auth::strategies::oauth2::AuthorityParams;
use oxidauth_kernel::{auth::authenticate_or_register::OAuth2Profile, error::BoxedError};

use super::GoogleProfile;

pub async fn retrieve_google_profile(
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
            error!("ERROR IN RETRIEVE GOOGLE PROFILE TEXT {}", err);

            err
        })?;

    let response_text = profile_response
        .text()
        .await?;

    let profile = serde_json::from_str::<GoogleProfile>(&response_text).map_err(|err| {
        error!(
            "ERROR IN RETRIEVE GOOGLE PROFILE TEXT JSON {}, {:?}",
            err, response_text
        );

        err
    })?;

    Ok(OAuth2Profile {
        email: profile.email,
        given_name: profile.given_name,
        family_name: profile.family_name,
    })
}
