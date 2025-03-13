use reqwest::header::AUTHORIZATION;

use crate::auth::strategies::oauth2::AuthorityParams;
use oxidauth_kernel::{auth::authenticate_or_register::OAuth2Profile, error::BoxedError};

use super::GoogleProfile;

pub async fn retrieve_profile(
    access_token: String,
    authority_params: &AuthorityParams,
) -> Result<OAuth2Profile, BoxedError> {
    let bearer_token = format!("Bearer {}", &access_token);

    let profile: GoogleProfile = reqwest::Client::new()
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
        email: profile.email,
        given_name: profile.given_name,
        family_name: profile.family_name,
    })
}
