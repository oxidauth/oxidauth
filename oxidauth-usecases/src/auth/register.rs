use async_trait::async_trait;
use base64::prelude::*;
use chrono::DateTime;
use oxidauth_kernel::{
    auth::{
        Registrar,
        register::{RegisterParams, RegisterResponse},
    },
    authorities::{Authority, AuthorityNotFoundError, AuthorityStrategy, NbfOffset},
    error::BoxedError,
    jwt::{DurationDirection, Jwt, epoch_from_now},
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
    service::Service,
};
use oxidauth_repository::{
    auth::tree::{PermissionSearch, PermissionTreeQuery},
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::{CreateRefreshToken, InsertRefreshTokenQuery},
    user_authorities::insert_user_authority::InsertUserAuthorityQuery,
    users::insert_user::InsertUserQuery,
};

use crate::auth::strategies;

pub struct RegisterUseCase<T, U, A, P, M, R>
where
    T: SelectAuthorityByClientKeyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    authority_by_client_key: T,
    users: U,
    user_authorities: A,
    permission_tree: P,
    private_keys: M,
    refresh_tokens: R,
}

impl<T, U, A, P, M, R> RegisterUseCase<T, U, A, P, M, R>
where
    T: SelectAuthorityByClientKeyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    pub fn new(
        authority_by_client_key: T,
        users: U,
        user_authorities: A,
        permission_tree: P,
        private_keys: M,
        refresh_tokens: R,
    ) -> Self {
        Self {
            authority_by_client_key,
            users,
            user_authorities,
            permission_tree,
            private_keys,
            refresh_tokens,
        }
    }
}

#[async_trait]
impl<'a, T, U, A, P, M, R> Service<&'a RegisterParams> for RegisterUseCase<T, U, A, P, M, R>
where
    T: SelectAuthorityByClientKeyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    type Response = RegisterResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "register_usecase", skip(self))]
    async fn call(&self, params: &'a RegisterParams) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authority_by_client_key
            .call(&params.into())
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(params.client_key))?;

        let registrar = build_registrar(&authority).await?;

        let (user, user_authority) = registrar
            .register(params.params.clone())
            .await?;

        let user = self.users.call(&user).await?;

        self.user_authorities
            .call((user.id, &user_authority))
            .await?;

        // add default roles and permissions
        let permissions = self
            .permission_tree
            .call(&PermissionSearch::User(user.id))
            .await?
            .permissions;

        let private_key = self
            .private_keys
            .call(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let mut jwt_builder = Jwt::builder()
            .with_subject(user.id)
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
            .refresh_tokens
            .call(&CreateRefreshToken {
                user_id: user.id,
                authority_id: authority.id,
                expires_at: refresh_token_exp_at,
            })
            .await?;

        Ok(RegisterResponse {
            jwt,
            refresh_token: refresh_token.id,
            user_id: user.id,
        })
    }
}

pub async fn build_registrar(authority: &Authority) -> Result<Box<dyn Registrar>, BoxedError> {
    use AuthorityStrategy::*;

    match authority.strategy {
        UsernamePassword => strategies::username_password::registrar::new(authority).await,
        Oauth2 => strategies::oauth2::registrar::new(authority).await,
        SingleUseToken => unimplemented!(),
    }
}
