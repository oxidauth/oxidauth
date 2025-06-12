use async_trait::async_trait;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use boringauth::oath::TOTPBuilder;
use chrono::DateTime;
use oxidauth_kernel::{authorities::NbfOffset, jwt::DurationDirection};
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

pub use oxidauth_kernel::{
    auth::{
        Authenticator,
        authenticate::{AuthenticateParams, AuthenticateResponse, WebhookReq, WebhookRes},
    },
    authorities::{Authority, AuthorityNotFoundError, AuthorityStrategy, TotpSettings},
    error::BoxedError,
    jwt::{Jwt, epoch_from_now},
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
    service::Service,
    totp_secrets::{TOTPSecret, find_totp_secret_by_user_id::FindTOTPSecretByUserId},
    users::find_user_by_id::FindUserById,
};
use oxidauth_repository::{
    auth::tree::{PermissionSearch, PermissionTreeQuery},
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::{CreateRefreshToken, InsertRefreshTokenQuery},
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
    user_authorities::select_user_authorities_by_authority_id_and_user_identifier::{
        SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
        SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams,
    },
    users::select_user_by_id_query::SelectUserByIdQuery,
};

use crate::{auth::strategies::*, bootstrap::TOTP_VALIDATE_PERMISSION};

pub struct AuthenticateUseCase<T, U, P, M, R, S, UU>
where
    T: SelectAuthorityByClientKeyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    UU: SelectUserByIdQuery,
{
    authority_by_client_key: T,
    user_authority: U,
    permission_tree: P,
    private_keys: M,
    refresh_tokens: R,
    user_totp_secret: S,
    user_by_id: UU,
}

impl<T, U, P, M, R, S, UU> AuthenticateUseCase<T, U, P, M, R, S, UU>
where
    T: SelectAuthorityByClientKeyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    UU: SelectUserByIdQuery,
{
    pub fn new(
        authority_by_client_key: T,
        user_authority: U,
        permission_tree: P,
        private_keys: M,
        refresh_tokens: R,
        user_totp_secret: S,
        user_by_id: UU,
    ) -> Self {
        Self {
            authority_by_client_key,
            user_authority,
            permission_tree,
            private_keys,
            refresh_tokens,
            user_totp_secret,
            user_by_id,
        }
    }
}

#[async_trait]
impl<'a, T, U, P, M, R, S, UU> Service<&'a AuthenticateParams>
    for AuthenticateUseCase<T, U, P, M, R, S, UU>
where
    T: SelectAuthorityByClientKeyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    UU: SelectUserByIdQuery,
{
    type Response = AuthenticateResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "authenticate_usecase", skip(self))]
    async fn call(&self, params: &'a AuthenticateParams) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authority_by_client_key
            .call(&params.into())
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(params.client_key))?;

        let authenticator = build_authenticator(&authority).await?;

        let user_identifier = authenticator
            .user_identifier_from_request(&params.params)
            .await?;

        let user_authority_params =
            SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams {
                authority_id: authority.id,
                user_identifier: user_identifier.clone(),
            };

        let user_authority = self
            .user_authority
            .call(&user_authority_params)
            .await?;

        let _ = authenticator
            .authenticate(params.params.clone(), &authority, &user_authority)
            .await?;

        let private_key = self
            .private_keys
            .call(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let mut jwt_builder = Jwt::builder()
            .with_subject(user_authority.user_id)
            .with_issuer("oxidauth".to_owned());

        let setting = authority
            .settings
            .jwt_nbf_offset;

        if let NbfOffset::Enabled(value) = setting {
            jwt_builder = jwt_builder.with_not_before_from(value);
        };

        match authority.settings.totp {
            TotpSettings::Enabled {
                totp_ttl,
                webhook,
                webhook_key,
            } => {
                let user = self
                    .user_by_id
                    .call(&FindUserById {
                        user_id: user_authority.user_id,
                    })
                    .await?;

                info!("login requires 2FA");

                jwt_builder = jwt_builder
                    .with_expires_in(totp_ttl)
                    .with_entitlements(
                        authority
                            .settings
                            .entitlements_encoding,
                        &[TOTP_VALIDATE_PERMISSION.to_string()],
                    );

                // get the secret key for the user by id
                let secret_by_user_id: TOTPSecret = self
                    .user_totp_secret
                    .call(&FindTOTPSecretByUserId {
                        user_id: user_authority.user_id,
                    })
                    .await?;

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| "time is before 1970")?;

                let code = TOTPBuilder::new()
                    .ascii_key(&secret_by_user_id.secret)
                    .period(totp_ttl.as_secs() as u32)
                    .timestamp(now.as_secs() as i64)
                    .finalize()
                    .map_err(|err| format!("error generating totp: {:?}", err))?
                    .generate();

                let name = match (user.first_name, user.last_name) {
                    (None, None) => None,
                    (None, Some(last)) => Some(last),
                    (Some(first), None) => Some(first),
                    (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                };

                let email = user
                    .email
                    .ok_or("unable to send totp code - missing email")?;

                let webhook_params = WebhookReq {
                    webhook_key,
                    name,
                    email,
                    code,
                };

                let client = Client::new();
                let webhook_res: WebhookRes = client
                    .post(webhook)
                    .json(&webhook_params)
                    .send()
                    .await?
                    .json()
                    .await?;

                if !webhook_res.success {
                    return Err(format!("unable to send totp code to: {}", user_identifier,).into());
                }
            },
            TotpSettings::Disabled => {
                let permissions = self
                    .permission_tree
                    .call(&PermissionSearch::User(user_authority.user_id))
                    .await?
                    .permissions;

                jwt_builder = jwt_builder
                    .with_expires_in(authority.settings.jwt_ttl)
                    .with_entitlements(
                        authority
                            .settings
                            .entitlements_encoding,
                        &permissions,
                    );
            },
        }

        let refresh_token_exp_at = epoch_from_now(
            DurationDirection::Add,
            authority
                .settings
                .refresh_token_ttl,
        )
        .map_err(|err| format!("unable to calculate refresh_token_exp_at: {:?}", err))?;

        let refresh_token_exp_at = DateTime::from_timestamp(refresh_token_exp_at as i64, 0)
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
            .map_err(|err| format!("unable to build jwt: {:?}", err))?
            .encode(&private_key)
            .map_err(|err| format!("unable to encode jwt: {:?}", err))?;

        let response = AuthenticateResponse {
            jwt,
            refresh_token: refresh_token.id,
            user_id: user_authority.user_id,
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
        UsernamePassword => username_password::authenticator::new(authority).await,
        SingleUseToken => unimplemented!(),
        Oauth2 => oauth2::authenticator::new(authority).await,
    }
}
