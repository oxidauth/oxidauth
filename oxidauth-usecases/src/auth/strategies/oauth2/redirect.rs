use async_trait::async_trait;

use oxidauth_kernel::{
    auth::oauth2::redirect::{Oauth2RedirectParams, Oauth2RedirectResponse},
    authorities::{AuthorityNotFoundError, find_authority_by_client_key::*},
    error::BoxedError,
};
use oxidauth_repository::authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery;

use super::{AuthorityParams, OauthProviders};

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

    #[tracing::instrument(
        name = "find_authority_by_client_key_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a Oauth2RedirectParams,
    ) -> Result<Self::Response, Self::Error> {
        // Get the authority
        let authority = self
            .authorities
            .call(&FindAuthorityByClientKey {
                client_key: params.client_key,
            })
            .await?
            .ok_or_else(|| {
                AuthorityNotFoundError::client_key(params.client_key)
            })?;

        // Get the oauth flavor from params
        let oauth_params: AuthorityParams = authority.params.try_into()?;

        // construct the redirect url based on the oauth flavor
        let redirect_url = match oauth_params.flavor {
            OauthProviders::Google => oauth_params.redirect_url,
        };

        // Example Redirect: "https://accounts.google.com/o/oauth2/v2/auth?{}{}{}{}&response_type=code&include_granted_scopes=true",
        // let client_id = "client_id=127751927363-4l0710vnomm37imtagelivu0sn8rui3b.apps.googleusercontent.com&";
        // let response_url = "redirect_uri=http://localhost:8000/api/v1/auth/sso/oauth/auth_response&";
        // let scope = "scope=https://www.googleapis.com/auth/userinfo.profile&";
        // let hd = "";

        Ok(Oauth2RedirectResponse { redirect_url })
    }
}
