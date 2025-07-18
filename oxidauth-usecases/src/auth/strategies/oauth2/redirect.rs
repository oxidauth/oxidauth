use std::fmt::Error;

use argon2::{
    Argon2,
    password_hash::{Error as HashError, PasswordHasher, SaltString},
};
use async_trait::async_trait;
use rand_core::OsRng;
use uuid::Uuid;

use oxidauth_kernel::{
    auth::oauth2::redirect::{
        Oauth2RedirectParams, Oauth2RedirectResponse, ParseOauth2RedirectUrlError,
    },
    authorities::{AuthorityNotFoundError, find_authority_by_client_key::*},
    error::BoxedError,
};
use oxidauth_repository::authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery;

use super::{AuthorityParams, OAuthFlavors};

pub struct Oauth2RedirectUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    authorities: T,
}

impl<T> Oauth2RedirectUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a Oauth2RedirectParams> for Oauth2RedirectUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    type Response = Oauth2RedirectResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_authority_by_client_key_usecase", skip(self))]
    async fn call(&self, params: &'a Oauth2RedirectParams) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authorities
            .call(&FindAuthorityByClientKey {
                client_key: params.client_key,
            })
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(params.client_key))?;

        let oauth_params: AuthorityParams = authority.params.try_into()?;

        let Ok(state_hash) = state_hasher(authority.client_key) else {
            let err = Error;
            return Err(Box::new(err));
        };

        let redirect_url = match oauth_params.flavor {
            OAuthFlavors::Google => {
                let mut redirect_url = oauth_params.redirect_url;

                let redirect_url = redirect_url
                    .query_pairs_mut()
                    .append_pair("login_hint", &params.email)
                    .append_pair("response_type", "code")
                    .append_pair("include_granted_scopes", "true")
                    .append_pair("state", state_hash.as_str())
                    .finish();

                redirect_url.to_owned()
            },
            OAuthFlavors::Microsoft => {
                let mut redirect_url = oauth_params.redirect_url;

                let redirect_url = redirect_url
                    .query_pairs_mut()
                    .append_pair("login_hint", &params.email)
                    .append_pair("response_type", "code")
                    .append_pair("response_mode", "query")
                    .append_pair("state", state_hash.as_str())
                    .finish();

                redirect_url.to_owned()
            },
        };

        Ok(Oauth2RedirectResponse { redirect_url })
    }
}

pub fn state_hasher(client_id: Uuid) -> Result<String, HashError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(&client_id.into_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}
