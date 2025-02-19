use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use oxidauth_kernel::{
    auth::oauth2::authenticate::{
        Oauth2Authenticate, Oauth2AuthenticateResponse,
    },
    authorities::{
        AuthorityNotFoundError,
        find_authority_by_client_key::FindAuthorityByClientKey,
    },
    error::BoxedError,
    service::Service,
};
use oxidauth_repository::authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery;

use super::AuthorityParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoogleExchangeTokenReq {
    code: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    grant_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoogleExchangeTokenRes {
    access_token: String,
    expires_in: u32,
    scope: String,
    token_type: String,
    id_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoogleProfile {
    name: String,
    given_name: String,
    family_name: String,
    picture: String,
    id: String,
    email: String,
    verified_email: bool,
}

pub struct Oauth2AuthenticateUseCase<A>
where
    A: SelectAuthorityByClientKeyQuery,
{
    authority_by_client_key: A,
}

impl<A> Oauth2AuthenticateUseCase<A>
where
    A: SelectAuthorityByClientKeyQuery,
{
    pub fn new(authority_by_client_key: A) -> Self {
        Self {
            authority_by_client_key,
        }
    }
}

#[async_trait]
impl<'a, A> Service<&'a Oauth2Authenticate> for Oauth2AuthenticateUseCase<A>
where
    A: SelectAuthorityByClientKeyQuery,
{
    type Response = Oauth2AuthenticateResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "oauth2 authenticate ", skip(self))]
    async fn call(
        &self,
        params: &'a Oauth2Authenticate,
    ) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authority_by_client_key
            .call(&FindAuthorityByClientKey {
                client_key: params.client_key,
            })
            .await?
            .ok_or_else(|| {
                AuthorityNotFoundError::client_key(params.client_key)
            })?;

        let authority_params: AuthorityParams = authority.params.try_into()?;

        let json = GoogleExchangeTokenReq {
            code: params.code.to_owned(),
            client_id: authority_params.oauth2_id,
            client_secret: authority_params.oauth2_secret,
            redirect_uri: authority_params
                .redirect_url
                .to_string(),
            grant_type: String::from("authorization_code"),
        };

        println!("JSON :: {:?}", json);

        let exchange: GoogleExchangeTokenRes = reqwest::Client::new()
            .post("https://oauth2.googleapis.com/token")
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .form(&json)
            .send()
            .await
            .map_err(|err| {
                println!(
                    "ERROR WITH REQWEST CALL :: {:?}",
                    err.to_string()
                );
                err.to_string()
            })?
            .json()
            .await
            .map_err(|err| {
                println!(
                    "ERROR PARSING JSON {:?}",
                    err.to_string(),
                );
                err.to_string()
            })?;

        println!(
            "ACCESS TOKEN RECEIVED:: {:?}",
            exchange
        );

        let mut bearer_token = String::from("Bearer ");
        bearer_token.push_str(&exchange.access_token);
        println!(
            "MADE THE BEARER TOKEN {:?}",
            bearer_token.clone()
        );

        let profile: GoogleProfile = reqwest::Client::new()
            .get("https://www.googleapis.com/userinfo/v2/me")
            .header(AUTHORIZATION, bearer_token)
            .send()
            .await
            .map_err(|err| {
                println!(
                    "GOOGLE RESPONSE ERROR 1 :: {:?}",
                    err
                );
                err.to_string()
            })?
            .json()
            .await
            .map_err(|err| err.to_string())?;

        println!("PROFILE INFO:: {:?}", profile);

        let profile = serde_json::to_value(profile)?.into();

        Ok(Oauth2AuthenticateResponse { profile })
    }
}
