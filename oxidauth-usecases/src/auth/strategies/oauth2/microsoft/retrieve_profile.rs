use reqwest::header::AUTHORIZATION;

use crate::auth::strategies::oauth2::AuthorityParams;
use oxidauth_kernel::{auth::authenticate_or_register::OAuth2Profile, error::BoxedError};

use super::MicrosoftProfile;

pub async fn retrieve_microsoft_profile(
    access_token: String,
    authority_params: &AuthorityParams,
) -> Result<OAuth2Profile, BoxedError> {
    let bearer_token = format!("Bearer {}", &access_token);

    let profile: MicrosoftProfile = reqwest::Client::new()
        .get(
            authority_params
                .profile_url
                .clone(),
        )
        .header(AUTHORIZATION, bearer_token)
        .send()
        .await?
        .json()
        .await?;

    Ok(OAuth2Profile {
        email: profile.mail,
        given_name: profile.givenName,
        family_name: profile.surname,
    })
}
