use oxidauth_kernel::{auth::authenticate_or_register::OAuth2Profile, error::BoxedError};
use reqwest::header::AUTHORIZATION;

use super::GoogleProfile;
use crate::auth::strategies::oauth2::AuthorityParams;

pub async fn retrieve_google_profile(
    access_token: String,
    authority_params: &AuthorityParams,
) -> Result<OAuth2Profile, BoxedError> {
    let bearer_token = format!("Bearer {}", access_token);

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

    // OXA-000073: `verified_email` is enforced here, not discarded
    profile
        .into_oauth_profile()
        .map_err(Into::into)
}
