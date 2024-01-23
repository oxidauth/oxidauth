use std::time::{Duration, SystemTime};

use base64::prelude::{Engine, BASE64_STANDARD};

use async_trait::async_trait;

use chrono::DateTime;
use oxidauth_kernel::auth::authenticate::AuthenticateResponse;
use oxidauth_kernel::auth::tree::PermissionSearch;
use oxidauth_kernel::authorities::find_authority_by_id::FindAuthorityById;
use oxidauth_kernel::jwt::{epoch_from_time, Jwt, epoch_from_now};
use oxidauth_kernel::private_keys::find_most_recent_private_key::FindMostRecentPrivateKey;
use oxidauth_kernel::refresh_tokens::create_refresh_token::CreateRefreshToken;
use oxidauth_kernel::refresh_tokens::find_refresh_token_by_id::FindRefreshTokenById;
use oxidauth_kernel::user_authorities::find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityId;
use oxidauth_kernel::{refresh_tokens::exchange_refresh_token::*, error::BoxedError};
use oxidauth_repository::auth::tree::PermissionTreeQuery;
use oxidauth_repository::private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery;
use oxidauth_repository::refresh_tokens::insert_refresh_token::InsertRefreshTokenQuery;
use oxidauth_repository::refresh_tokens::select_refresh_token_by_id::SelectRefreshTokenByIdQuery;
use oxidauth_repository::user_authorities::select_user_authority_by_user_id_and_authority_id::SelectUserAuthorityByUserIdAndAuthorityIdQuery;
use oxidauth_repository::authorities::select_authority_by_id::SelectAuthorityByIdQuery;

pub struct ExchangeRefreshTokenUseCase<T, I, U, A, P, K>
where
    T: SelectRefreshTokenByIdQuery,
    I: InsertRefreshTokenQuery,
    U: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    A: SelectAuthorityByIdQuery,
    P: PermissionTreeQuery,
    K: SelectMostRecentPrivateKeyQuery,
{
    refresh_tokens: T,
    insert_refresh_tokens: I,
    user_authorities: U,
    authorities: A,
    permission_tree: P,
    private_keys: K,
}

impl<T, I, U, A, P, K> ExchangeRefreshTokenUseCase<T, I, U, A, P, K>
where
    T: SelectRefreshTokenByIdQuery,
    I: InsertRefreshTokenQuery,
    U: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    A: SelectAuthorityByIdQuery,
    P: PermissionTreeQuery,
    K: SelectMostRecentPrivateKeyQuery,
{
    pub fn new(
        refresh_tokens: T,
        insert_refresh_tokens: I,
        user_authorities: U,
        authorities: A,
        permission_tree: P,
        private_keys: K,
    ) -> Self {
        Self {
            refresh_tokens,
            insert_refresh_tokens,
            user_authorities,
            authorities,
            permission_tree,
            private_keys,
        }
    }
}

#[async_trait]
impl<'a, T, I, U, A, P, K> Service<&'a ExchangeRefreshToken>
    for ExchangeRefreshTokenUseCase<T, I, U, A, P, K>
where
    T: SelectRefreshTokenByIdQuery,
    I: InsertRefreshTokenQuery,
    U: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    A: SelectAuthorityByIdQuery,
    P: PermissionTreeQuery,
    K: SelectMostRecentPrivateKeyQuery,
{
    type Response = AuthenticateResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "exchange_refresh_token_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a ExchangeRefreshToken,
    ) -> Result<Self::Response, Self::Error> {
        let RefreshToken {
            user_id,
            authority_id,
            expires_at,
            ..
        } = self
            .refresh_tokens
            .call(&FindRefreshTokenById {
                refresh_token_id: req.refresh_token,
            })
            .await?;

        let now = epoch_from_time(SystemTime::now()).map_err(|err| {
            format!(
                "error getting epoch time from system time: {:?}",
                err
            )
        })?;

        if expires_at.timestamp() < now as i64 {
            return Err("refresh token has expired".into());
        }

        let authority_id = self
            .user_authorities
            .call(
                &FindUserAuthorityByUserIdAndAuthorityId {
                    user_id,
                    authority_id,
                },
            )
            .await?
            .authority
            .id;

        let authority = self
            .authorities
            .call(&FindAuthorityById { authority_id })
            .await?;

        let permissions = self
            .permission_tree
            .call(&PermissionSearch::User(
                user_id,
            ))
            .await?
            .permissions;

        let private_key = self
            .private_keys
            .call(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let jwt = Jwt::new()
            .with_subject(user_id)
            .with_issuer("oxidauth".to_owned())
            .with_expires_in(authority.settings.jwt_ttl)
            .with_entitlements(permissions)
            .with_not_before_from(Duration::from_secs(0))
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
            .insert_refresh_tokens
            .call(&CreateRefreshToken {
                user_id,
                authority_id: authority.id,
                expires_at: refresh_token_exp_at,
            })
            .await?;

        Ok(AuthenticateResponse {
            jwt,
            refresh_token: refresh_token.id,
        })
    }
}
