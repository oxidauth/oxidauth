use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use oxidauth_kernel::{
    JsonValue,
    auth::{
        Authenticator,
        oauth2::authenticate::{OAuth2AuthenticateParams, OAuth2AuthenticateResponse},
    },
    authorities::{
        Authority, AuthorityNotFoundError, UserAuthority,
        find_authority_by_client_key::FindAuthorityByClientKey,
    },
    error::BoxedError,
    service::Service,
};

use super::{AuthorityParams, OAuth2};

#[derive(Clone, Deserialize)]
pub struct AuthenticateParams {
    pub username: String,
    pub access_token: String,
}

impl TryFrom<JsonValue> for AuthenticateParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}

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

impl TryFrom<JsonValue> for GoogleProfile {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}

#[async_trait]
impl Authenticator for OAuth2 {
    #[tracing::instrument(name = "oauth2 authenticate", skip(self))]
    async fn authenticate(
        &self,
        authenticate_params: JsonValue,
        authority: &Authority,
        user_authority: &UserAuthority,
    ) -> Result<(), BoxedError> {
        let authenticate_params: OAuth2AuthenticateParams = authenticate_params.try_into()?;
        let authority_params: AuthorityParams = authority
            .params
            .clone()
            .try_into()?;

        let json = GoogleExchangeTokenReq {
            code: authenticate_params
                .code
                .to_owned(),
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
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&json)
            .send()
            .await
            .map_err(|err| {
                println!("ERROR WITH REQWEST CALL :: {:?}", err.to_string());
                err.to_string()
            })?
            .json()
            .await
            .map_err(|err| {
                println!("ERROR PARSING JSON {:?}", err.to_string(),);
                err.to_string()
            })?;

        println!("ACCESS TOKEN RECEIVED:: {:?}", exchange);

        let mut bearer_token = String::from("Bearer ");
        bearer_token.push_str(&exchange.access_token);
        println!("MADE THE BEARER TOKEN {:?}", bearer_token.clone());

        let profile: GoogleProfile = reqwest::Client::new()
            .get("https://www.googleapis.com/userinfo/v2/me")
            .header(AUTHORIZATION, bearer_token)
            .send()
            .await
            .map_err(|err| {
                println!("GOOGLE RESPONSE ERROR 1 :: {:?}", err);
                err.to_string()
            })?
            .json()
            .await
            .map_err(|err| err.to_string())?;

        println!("PROFILE INFO:: {:?}", profile);

        Ok(())
    }
}
