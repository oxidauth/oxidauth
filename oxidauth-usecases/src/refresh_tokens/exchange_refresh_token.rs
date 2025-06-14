use std::time::SystemTime;

use base64::prelude::{BASE64_STANDARD, Engine};

use async_trait::async_trait;

use chrono::DateTime;
use oxidauth_kernel::auth::{authenticate::AuthenticateResponse, tree::PermissionSearch};
use oxidauth_kernel::authorities::{NbfOffset, find_authority_by_id::FindAuthorityById};
use oxidauth_kernel::jwt::{DurationDirection, Jwt, epoch_from_now, epoch_from_time};
use oxidauth_kernel::private_keys::find_most_recent_private_key::FindMostRecentPrivateKey;
use oxidauth_kernel::refresh_tokens::create_refresh_token::CreateRefreshToken;
use oxidauth_kernel::refresh_tokens::{
    delete_refresh_token::DeleteRefreshToken, find_refresh_token_by_id::FindRefreshTokenById,
};
use oxidauth_kernel::user_authorities::find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityId;
use oxidauth_kernel::{error::BoxedError, refresh_tokens::exchange_refresh_token::*};
use oxidauth_repository::auth::tree::PermissionTreeQuery;
use oxidauth_repository::authorities::select_authority_by_id::SelectAuthorityByIdQuery;
use oxidauth_repository::private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery;
use oxidauth_repository::refresh_tokens::{
    delete_refresh_token::DeleteRefreshTokenQuery, insert_refresh_token::InsertRefreshTokenQuery,
    select_refresh_token_by_id::SelectRefreshTokenByIdQuery,
};
use oxidauth_repository::user_authorities::select_user_authority_by_user_id_and_authority_id::SelectUserAuthorityByUserIdAndAuthorityIdQuery;

pub struct ExchangeRefreshTokenUseCase<T, I, U, A, P, K, D>
where
    T: SelectRefreshTokenByIdQuery,
    I: InsertRefreshTokenQuery,
    U: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    A: SelectAuthorityByIdQuery,
    P: PermissionTreeQuery,
    K: SelectMostRecentPrivateKeyQuery,
    D: DeleteRefreshTokenQuery,
{
    refresh_tokens: T,
    insert_refresh_tokens: I,
    user_authorities: U,
    authorities: A,
    permission_tree: P,
    private_keys: K,
    delete_refresh_tokens: D,
}

impl<T, I, U, A, P, K, D> ExchangeRefreshTokenUseCase<T, I, U, A, P, K, D>
where
    T: SelectRefreshTokenByIdQuery,
    I: InsertRefreshTokenQuery,
    U: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    A: SelectAuthorityByIdQuery,
    P: PermissionTreeQuery,
    K: SelectMostRecentPrivateKeyQuery,
    D: DeleteRefreshTokenQuery,
{
    pub fn new(
        refresh_tokens: T,
        insert_refresh_tokens: I,
        user_authorities: U,
        authorities: A,
        permission_tree: P,
        private_keys: K,
        delete_refresh_tokens: D,
    ) -> Self {
        Self {
            refresh_tokens,
            insert_refresh_tokens,
            user_authorities,
            authorities,
            permission_tree,
            private_keys,
            delete_refresh_tokens,
        }
    }
}

#[async_trait]
impl<'a, T, I, U, A, P, K, D> Service<&'a ExchangeRefreshToken>
    for ExchangeRefreshTokenUseCase<T, I, U, A, P, K, D>
where
    T: SelectRefreshTokenByIdQuery,
    I: InsertRefreshTokenQuery,
    U: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    A: SelectAuthorityByIdQuery,
    P: PermissionTreeQuery,
    K: SelectMostRecentPrivateKeyQuery,
    D: DeleteRefreshTokenQuery,
{
    type Response = AuthenticateResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "exchange_refresh_token_usecase", skip(self))]
    async fn call(&self, req: &'a ExchangeRefreshToken) -> Result<Self::Response, Self::Error> {
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

        let now = epoch_from_time(SystemTime::now())
            .map_err(|err| format!("error getting epoch time from system time: {:?}", err))?;

        if expires_at.timestamp() < now as i64 {
            self.delete_refresh_tokens
                .call(&DeleteRefreshToken {
                    refresh_token_id: req.refresh_token,
                })
                .await?;
            return Err("refresh token has expired".into());
        }

        let authority_id = self
            .user_authorities
            .call(&FindUserAuthorityByUserIdAndAuthorityId {
                user_id,
                authority_id,
            })
            .await?
            .authority
            .id;

        let authority = self
            .authorities
            .call(&FindAuthorityById { authority_id })
            .await?;

        let permissions = self
            .permission_tree
            .call(&PermissionSearch::User(user_id))
            .await?
            .permissions;

        let private_key = self
            .private_keys
            .call(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let mut jwt_builder = Jwt::builder()
            .with_subject(user_id)
            .with_issuer("oxidauth".to_owned())
            .with_expires_in(authority.settings.jwt_ttl)
            .with_entitlements(
                authority
                    .settings
                    .entitlements_encoding,
                &permissions,
            );

        if let NbfOffset::Enabled(value) = authority
            .settings
            .jwt_nbf_offset
        {
            jwt_builder = jwt_builder.with_not_before_from(value);
        };

        let jwt = jwt_builder
            .build()
            .map_err(|err| format!("unable to build jwt: {:?}", err))?
            .encode(&private_key)
            .map_err(|err| format!("unable to encode jwt: {:?}", err))?;

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
            .insert_refresh_tokens
            .call(&CreateRefreshToken {
                user_id,
                authority_id: authority.id,
                expires_at: refresh_token_exp_at,
            })
            .await?;

        self.delete_refresh_tokens
            .call(&DeleteRefreshToken {
                refresh_token_id: req.refresh_token,
            })
            .await?;

        Ok(AuthenticateResponse {
            jwt,
            refresh_token: refresh_token.id,
            user_id,
        })
    }
}
