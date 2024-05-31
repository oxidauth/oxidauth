use std::time::Duration;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use tracing::info;
use chrono::DateTime;
use async_trait::async_trait;
use oxidauth_kernel::{
    auth::{
        authenticate::{AuthenticateParams, AuthenticateResponse},
        Authenticator,
    },
    authorities::{Authority, AuthorityNotFoundError, AuthorityStrategy},
    error::BoxedError,
    jwt::{epoch_from_now, Jwt},
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
    service::Service,
    totp::generate::{GenerateTOTP, GenerateTOTPTrait},
};
use oxidauth_repository::{
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    user_authorities::select_user_authorities_by_authority_id_and_user_identifier::{
        SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
        SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams
    },
    auth::tree::{PermissionSearch, PermissionTreeQuery},
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::{
        CreateRefreshToken,
        InsertRefreshTokenQuery,
    },
};

use crate::auth::strategies::*;

pub struct AuthenticateUseCase<T, U, P, M, R, S>
where
    T: SelectAuthorityByClientKeyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: GenerateTOTPTrait,
{
    authority_by_client_key: T,
    user_authority: U,
    permission_tree: P,
    private_keys: M,
    refresh_tokens: R,
    generate_totp_service: S,
}

impl<T, U, P, M, R, S> AuthenticateUseCase<T, U, P, M, R, S>
where
    T: SelectAuthorityByClientKeyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: GenerateTOTPTrait,
{
    pub fn new(
        authority_by_client_key: T,
        user_authority: U,
        permission_tree: P,
        private_keys: M,
        refresh_tokens: R,
        generate_totp_service: S,
    ) -> Self {
        Self {
            authority_by_client_key,
            user_authority,
            permission_tree,
            private_keys,
            refresh_tokens,
            generate_totp_service,
        }
    }
}

#[async_trait]
impl<'a, T, U, P, M, R, S> Service<&'a AuthenticateParams>
    for AuthenticateUseCase<T, U, P, M, R, S>
where
    T: SelectAuthorityByClientKeyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: GenerateTOTPTrait,
{
    type Response = AuthenticateResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "authenticate_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a AuthenticateParams,
    ) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authority_by_client_key
            .call(&params.into())
            .await?
            .ok_or_else(|| {
                AuthorityNotFoundError::client_key(params.client_key)
            })?;

        let authenticator = build_authenticator(&authority).await?;

        let user_identifier = authenticator
            .user_identifier_from_request(&params.params)
            .await?;

        let user_authority_params =
            SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams {
                authority_id: authority.id,
                user_identifier,
            };

        let user_authority = self
            .user_authority
            .call(&user_authority_params)
            .await?;

        let _ = authenticator
            .authenticate(
                params.params.clone(),
                &user_authority,
            )
            .await?;

        let private_key = self
            .private_keys
            .call(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let mut jwt_builder = Jwt::builder()
            .with_subject(user_authority.user_id)
            .with_issuer("oxidauth".to_owned())
            .with_not_before_from(Duration::from_secs(0));

        // CHECK FOR 2FA REQUIREMENT AND START 2FA FLOW ---------
        if authority.settings.require_2fa {
            info!("login requires 2FA");

            // start the totp generate process (creates code, sends email)
            let _ = self
                .generate_totp_service
                .call(&GenerateTOTP {
                    user_id: user_authority.user_id,
                })
                .await;

            // Permission for viewing just the email code form in frontend
            const TOTP_PERMISSION: &str = "oxidauth:totp_code:validate";

            jwt_builder = jwt_builder
                .with_expires_in(
                    authority
                        .settings
                        .totp_jwt_ttl,
                )
                .with_entitlements(vec![
                    TOTP_PERMISSION.to_string()
                ]);
        } else {
            let permissions = self
                .permission_tree
                .call(&PermissionSearch::User(
                    user_authority.user_id,
                ))
                .await?
                .permissions;

            jwt_builder = jwt_builder
                .with_expires_in(authority.settings.jwt_ttl)
                .with_entitlements(permissions);
        }

        let refresh_token_exp_at = epoch_from_now(
            authority
                .settings
                .refresh_token_ttl,
        )
        .map_err(|err| {
            format!(
                "unable to calculate refresh_token_exp_at: {:?}",
                err
            )
        })?;

        let refresh_token_exp_at =
            DateTime::from_timestamp(refresh_token_exp_at as i64, 0)
                .ok_or("unable to convert refresh_token_exp_at to DateTime")?;

        let refresh_token = self
            .refresh_tokens
            .call(&CreateRefreshToken {
                user_id: user_authority.user_id,
                authority_id: authority.id,
                expires_at: refresh_token_exp_at,
            })
            .await?;

        let jwt = jwt_builder
            .build()
            .map_err(|err| {
                format!(
                    "unable to build jwt: {:?}",
                    err
                )
            })?
            .encode(&private_key)
            .map_err(|err| {
                format!(
                    "unable to encode jwt: {:?}",
                    err
                )
            })?;

        let response = AuthenticateResponse {
            jwt,
            refresh_token: refresh_token.id,
        };

        Ok(response)
    }
}

/// build_authenticator hydrates and returns an Authenticator
/// It takes the Authority (which has settings/params) and the strategy
/// and returns a Box<dyn Authenticator>
pub async fn build_authenticator(
    authority: &Authority,
) -> Result<Box<dyn Authenticator>, BoxedError> {
    use AuthorityStrategy::*;

    match authority.strategy {
        UsernamePassword => {
            username_password::authenticator::new(authority).await
        },
        SingleUseToken => unimplemented!(),
    }
}
