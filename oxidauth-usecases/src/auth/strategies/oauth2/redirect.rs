use async_trait::async_trait;

use oxidauth_kernel::{
    auth::oauth2::redirect::{Oauth2RedirectParams, Oauth2RedirectResponse},
    authorities::{AuthorityNotFoundError, find_authority_by_client_key::*},
    error::BoxedError,
};
use oxidauth_repository::authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery;
use tracing_log::log::info;

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
        info!("Got to redirect call");

        let authority = self
            .authorities
            .call(&FindAuthorityByClientKey {
                client_key: params.client_key,
            })
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(params.client_key))?;

        info!("Got authority");

        let oauth_params: AuthorityParams = authority.params.try_into()?;

        info!("Can get oauth params, {:?}", oauth_params);

        let redirect_url = match oauth_params.flavor {
            OAuthFlavors::Google => {
                let mut redirect_url = oauth_params.redirect_url;

                let redirect_url = redirect_url
                    .query_pairs_mut()
                    .append_pair("login_hint", &params.email)
                    .append_pair("response_type", "code")
                    .append_pair("include_granted_scopes", "true")
                    .finish();

                redirect_url.to_owned()
            },
        };

        Ok(Oauth2RedirectResponse { redirect_url })
    }
}
