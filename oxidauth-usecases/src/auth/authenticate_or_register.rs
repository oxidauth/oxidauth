use async_trait::async_trait;
use serde::Serialize;
use std::sync::Arc;

use oxidauth_kernel::{
    JsonValue,
    auth::{
        authenticate::AuthenticateParams, authenticate_or_register::*, register::RegisterParams,
    },
    authorities::find_authority_by_client_key::FindAuthorityByClientKey,
    error::BoxedError,
    service::Service,
    users::UserKind,
};
use oxidauth_repository::{
    auth::tree::PermissionTreeQuery,
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::InsertRefreshTokenQuery,
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
    user_authorities::{
        insert_user_authority::InsertUserAuthorityQuery,
        select_user_authorities_by_authority_id_and_user_identifier::SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    },
    users::{insert_user::InsertUserQuery, select_user_by_id_query::SelectUserByIdQuery},
};

use crate::auth::{
    strategies::oauth2::google::{exchange_token, retrieve_profile},
    strategies::oauth2::registrar::Oauth2RegisterParams,
};

use super::{
    authenticate::AuthenticateUseCase,
    register::RegisterUseCase,
    strategies::oauth2::{AuthorityParams, OAuthFlavors},
};

pub struct AuthenticateOrRegisterUseCase<A, M, P, R, S, T, UI, U, UU>
where
    A: InsertUserAuthorityQuery,
    M: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    UI: InsertUserQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    UU: SelectUserByIdQuery,
{
    authenticate: Arc<AuthenticateUseCase<T, U, P, M, R, S, UU>>,
    register: Arc<RegisterUseCase<T, UI, A, P, M, R>>,
    authorities: T,
}

impl<A, M, P, R, S, T, UI, U, UU> AuthenticateOrRegisterUseCase<A, M, P, R, S, T, UI, U, UU>
where
    A: InsertUserAuthorityQuery,
    M: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    UI: InsertUserQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    UU: SelectUserByIdQuery,
{
    pub fn new(
        authenticate: Arc<AuthenticateUseCase<T, U, P, M, R, S, UU>>,
        register: Arc<RegisterUseCase<T, UI, A, P, M, R>>,
        authorities: T,
    ) -> Self {
        Self {
            authenticate,
            register,
            authorities,
        }
    }

    async fn fetch_profile(
        &self,
        authority_params: &AuthorityParams,
        authenticate_params: &OAuth2AuthenticateParams,
    ) -> Result<OAuth2Profile, BoxedError> {
        match authority_params.flavor {
            OAuthFlavors::Google => {
                let code = authenticate_params
                    .code
                    .clone();

                let access_token = exchange_token(code, &authority_params).await?;

                retrieve_profile(access_token, &authority_params).await
            },
        }
    }
}

#[async_trait]
impl<'a, A, M, P, R, S, T, UI, U, UU> Service<&'a AuthenticateOrRegisterParams>
    for AuthenticateOrRegisterUseCase<A, M, P, R, S, T, UI, U, UU>
where
    A: InsertUserAuthorityQuery,
    M: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    UI: InsertUserQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    UU: SelectUserByIdQuery,
{
    type Response = AuthenticateOrRegisterResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "authenticate_or_register_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a AuthenticateOrRegisterParams,
    ) -> Result<Self::Response, Self::Error> {
        // GET AUTHORITY
        let Some(authority) = self
            .authorities
            .call(&FindAuthorityByClientKey {
                client_key: params.client_key,
            })
            .await?
        else {
            return Err(format!("could not find authority by client key").into());
        };

        // GET PARAMS
        let authority_params: AuthorityParams = authority.params.try_into()?;
        let authenticate_params: OAuth2AuthenticateParams = params
            .params
            .clone()
            .try_into()?;

        let profile = self
            .fetch_profile(&authority_params, &authenticate_params)
            .await?;

        #[derive(Debug, Serialize)]
        struct AuthParams {
            email: String,
        }

        let auth_params = serde_json::to_value(AuthParams {
            email: profile.email.clone(),
        })?;

        let authenticated = self
            .authenticate
            .call(&AuthenticateParams {
                client_key: params.client_key,
                params: JsonValue::new(auth_params),
            })
            .await;

        match authenticated {
            Ok(auth) => {
                return Ok(AuthenticateOrRegisterResponse {
                    jwt: auth.jwt,
                    refresh_token: auth.refresh_token,
                    client_base: authority_params.client_base_url,
                    email: profile.email.clone(),
                    given_name: profile.given_name.clone(),
                    family_name: profile.family_name.clone(),
                    user_id: auth.user_id,
                });
            },
            Err(err) => {
                if err
                    .to_string()
                    .contains("user authority not found:")
                {
                    let reg_params = Oauth2RegisterParams {
                        first_name: Some(profile.given_name.clone()),
                        last_name: Some(profile.family_name.clone()),
                        email: Some(profile.email.clone()),
                        username: profile.email.clone(),
                        kind: Some(UserKind::Human),
                    };

                    let reg_params_json = serde_json::to_value(reg_params)?;

                    let result = self
                        .register
                        .call(&RegisterParams {
                            client_key: params.client_key,
                            params: JsonValue::new(reg_params_json),
                        })
                        .await?;

                    let res = AuthenticateOrRegisterResponse {
                        jwt: result.jwt,
                        refresh_token: result.refresh_token,
                        client_base: authority_params.client_base_url,
                        email: profile.email.clone(),
                        given_name: profile.given_name,
                        family_name: profile.family_name,
                        user_id: result.user_id,
                    };

                    return Ok(res);
                }

                return Err(err);
            },
        }
    }
}
