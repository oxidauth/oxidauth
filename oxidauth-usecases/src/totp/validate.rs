use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use boringauth::oath::TOTPBuilder;
use chrono::DateTime;
use std::time::{SystemTime, UNIX_EPOCH};

use oxidauth_kernel::{
    auth::tree::PermissionSearch,
    authorities::{
        AuthorityNotFoundError, NbfOffset, TotpSettings,
        find_authority_by_client_key::FindAuthorityByClientKey,
    },
    error::BoxedError,
    jwt::{DurationDirection, Jwt, epoch_from_now},
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
    refresh_tokens::create_refresh_token::CreateRefreshToken,
    service::Service,
    totp::{validate::ValidateTOTP, TOTPValidationRes},
    totp_secrets::{
        find_totp_secret_by_user_id::FindTOTPSecretByUserId, TOTPSecret,
    },
};
use oxidauth_repository::{
    auth::tree::PermissionTreeQuery,
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::InsertRefreshTokenQuery,
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
};

pub struct ValidateTOTPUseCase<T, K, P, A, R>
where
    T: SelectTOTPSecrețByUserIdQuery,
    K: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    A: SelectAuthorityByClientKeyQuery,
    R: InsertRefreshTokenQuery,
{
    secret: T,
    private_keys: K,
    permission_tree: P,
    authority_by_client_key: A,
    refresh_tokens: R,
}

impl<T, K, P, A, R> ValidateTOTPUseCase<T, K, P, A, R>
where
    T: SelectTOTPSecrețByUserIdQuery,
    K: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    A: SelectAuthorityByClientKeyQuery,
    R: InsertRefreshTokenQuery,
{
    pub fn new(
        secret: T,
        private_keys: K,
        permission_tree: P,
        authority_by_client_key: A,
        refresh_tokens: R,
    ) -> Self {
        Self {
            secret,
            private_keys,
            permission_tree,
            authority_by_client_key,
            refresh_tokens,
        }
    }
}

#[async_trait]
impl<'a, T, K, P, A, R> Service<&'a ValidateTOTP>
    for ValidateTOTPUseCase<T, K, P, A, R>
where
    T: SelectTOTPSecrețByUserIdQuery,
    K: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    A: SelectAuthorityByClientKeyQuery,
    R: InsertRefreshTokenQuery,
{
    type Response = TOTPValidationRes;
    type Error = BoxedError;

    #[tracing::instrument(name = "validate_totp_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a ValidateTOTP,
    ) -> Result<Self::Response, Self::Error> {
        let user_id = req.user_id;

        let authority = self
            .authority_by_client_key
            .call(&FindAuthorityByClientKey {
                client_key: req.client_key,
            })
            .await?
            .ok_or_else(|| {
                AuthorityNotFoundError::client_key(req.client_key)
            })?;

        let totp_ttl = match authority.settings.totp {
            TotpSettings::Enabled { totp_ttl, .. } => totp_ttl,
            TotpSettings::Disabled => {
                return Err("totp_ttl missing because totp is disabled".into());
            },
        };

        // prepare TOTP secret params
        let secret_params = FindTOTPSecretByUserId { user_id };

        // get the secret key for the user by id
        let secret_by_user_id: TOTPSecret = self
            .secret
            .call(&secret_params)
            .await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "time is before 1970")?;

        let valid = TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(totp_ttl.as_secs() as u32)
            .timestamp(now.as_secs() as i64)
            .finalize()
            .unwrap()
            .is_valid(&req.code);

        if !valid {
            return Err("invalid totp code".into());
        }

        // BUILD JWT ----------------------------------------

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
            .with_entitlements(
                authority
                    .settings
                    .entitlements_encoding,
                &permissions,
            )
            .with_subject(user_id)
            .with_issuer("oxidauth".to_owned());

        if let NbfOffset::Enabled(value) = authority
            .settings
            .jwt_nbf_offset
        {
            jwt_builder = jwt_builder.with_not_before_from(value);
        };

        let refresh_token_exp_at = epoch_from_now(
            DurationDirection::Add,
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
