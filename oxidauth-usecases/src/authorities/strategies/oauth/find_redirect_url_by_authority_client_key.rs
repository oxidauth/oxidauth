// Construct redirect url from settings (how this is done might vary according to identity provider)
// front end to send user to resulting url to authenticate

use async_trait::async_trait;

use oxidauth_kernel::{
    auth::{
        FindRedirectUrlByAuthorityClientKey,
        FindRedirectUrlByAuthorityClientKeyError,
    },
    authorities::{
        find_authority_by_client_key::FindAuthorityByClientKey,
        find_authority_by_id::*, OauthSettings::*,
    },
};
use oxidauth_repository::authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery;

pub struct FindRedirectUrlByAuthorityClientKeyUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    authorities: T,
}

impl<T> FindRedirectUrlByAuthorityClientKeyUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindRedirectUrlByAuthorityClientKey>
    for FindRedirectUrlByAuthorityClientKeyUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    type Response = String;
    type Error = FindRedirectUrlByAuthorityClientKeyError;

    #[tracing::instrument(name = "find_authority_by_client_key", skip(self))]
    async fn call(
        &self,
        req: &'a FindRedirectUrlByAuthorityClientKey,
    ) -> Result<Self::Response, Self::Error> {
        let authority: Authority = self
            .authorities
            .call(&FindAuthorityByClientKey {
                client_key: req.authority_client_key,
            })
            .await
            .map_err(|_| FindRedirectUrlByAuthorityClientKeyError {})
            .unwrap()
            .unwrap();

        // construct redirect url
        match authority.settings.oauth {
            Enabled {
                client_id,
                response_url,
                scope,
                redirect_url_base,
            } => {
                let redirect_url = format!("{}?client_id={}&redirect_uri={}&scope={}&response_type=code", redirect_url_base, client_id, response_url, scope);

                return Ok(redirect_url);
            },
            Disabled => {
                return Err(FindRedirectUrlByAuthorityClientKeyError {});
            },
        }
    }
}
