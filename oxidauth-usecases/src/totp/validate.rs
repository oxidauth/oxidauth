use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use boringauth::oath::TOTPBuilder;
use chrono::DateTime;
use serde_json::json;
use std::time::Duration;

use oxidauth_kernel::{
    auth::tree::PermissionSearch, authorities::{find_authority_by_client_key::FindAuthorityByClientKey, AuthorityNotFoundError}, error::BoxedError, jwt::{epoch_from_now, Jwt}, private_keys::find_most_recent_private_key::FindMostRecentPrivateKey, refresh_tokens::create_refresh_token::CreateRefreshToken, service::Service, totp::{validate::ValidateTOTP, TOTPValidationRes}, totp_secrets::{
        select_totp_secret_by_user_id::SelectTOTPSecretByUserId, TOTPSecret,
    }, users::find_user_by_id::FindUserById
};
use oxidauth_repository::{
    auth::tree::PermissionTreeQuery, authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery, private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery, refresh_tokens::insert_refresh_token::InsertRefreshTokenQuery, totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery, user_authorities::select_user_authorities_by_authority_id_and_user_identifier::{SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery, SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams}, users::select_user_by_id_query::SelectUserByIdQuery
};

use crate::auth::authenticate::build_authenticator;

pub struct ValidateTOTPUseCase<T, K, P, U, A, R, I>
where
    T: SelectTOTPSecrețByUserIdQuery,
    K: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    A: SelectAuthorityByClientKeyQuery,
    R: InsertRefreshTokenQuery,
    I: SelectUserByIdQuery,
{
    secret: T,
    private_keys: K,
    permission_tree: P,
    user_authority: U,
    authority_by_client_key: A,
    refresh_tokens: R,
    user_by_id: I,
}

impl<T, K, P, U, A, R, I> ValidateTOTPUseCase<T, K, P, U, A, R, I>
where
    T: SelectTOTPSecrețByUserIdQuery,
    K: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    A: SelectAuthorityByClientKeyQuery,
    R: InsertRefreshTokenQuery,
    I: SelectUserByIdQuery,
{
    pub fn new(
        secret: T,
        private_keys: K,
        permission_tree: P,
        user_authority: U,
        authority_by_client_key: A,
        refresh_tokens: R,
        user_by_id: I,
    ) -> Self {
        Self {
            secret,
            private_keys,
            permission_tree,
            user_authority,
            authority_by_client_key,
            refresh_tokens,
            user_by_id,
        }
    }
}

#[async_trait]
impl<'a, T, K, P, U, A, R, I> Service<&'a ValidateTOTP>
    for ValidateTOTPUseCase<T, K, P, U, A, R, I>
where
    T: SelectTOTPSecrețByUserIdQuery,
    K: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    A: SelectAuthorityByClientKeyQuery,
    R: InsertRefreshTokenQuery,
    I: SelectUserByIdQuery,
{
    type Response = TOTPValidationRes;
    type Error = BoxedError;

    #[tracing::instrument(name = "validate_totp_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a ValidateTOTP,
    ) -> Result<Self::Response, Self::Error> {
        let user_id = req.user_id;

        // prepare TOTP secret params
        let secret_params = SelectTOTPSecretByUserId { user_id };

        // get the secret key for the user by id
        let secret_by_user_id: TOTPSecret = self
            .secret
            .call(&secret_params)
            .await?;

        let totp = TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(300)
            .finalize()
            .expect("Could not generate totp");

        println!(
            "TEST NEW CODE :::: {}",
            totp.generate()
        );

        let valid = boringauth::oath::TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(300)
            .finalize()
            .unwrap()
            .is_valid(&req.code);

        println!(
            "IS IT VALID ????? {}, {}",
            valid, req.code
        );

        if !valid {
            return Err("Code was not valid".into());
        }

        // BUILD JWT ----------------------------------------

        let authority_params = FindAuthorityByClientKey {
            client_key: req.client_key,
        };

        let authority = self
            .authority_by_client_key
            .call(&authority_params)
            .await?
            .ok_or_else(|| {
                AuthorityNotFoundError::client_key(req.client_key)
            })?;

        println!("got authority");

        let private_key = self
            .private_keys
            .call(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let permissions = self
            .permission_tree
            .call(&PermissionSearch::User(
                user_id,
            ))
            .await?
            .permissions;

        let mut jwt_builder = Jwt::builder()
            .with_expires_in(authority.settings.jwt_ttl)
            .with_entitlements(permissions)
            .with_subject(user_id)
            .with_issuer("oxidauth".to_owned())
            .with_not_before_from(Duration::from_secs(0));

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
                user_id,
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

        let response = TOTPValidationRes {
            jwt,
            refresh_token: refresh_token.id,
        };

        Ok(response)
    }
}
