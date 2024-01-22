use std::time::Duration;
use base64::{
    engine::{general_purpose, GeneralPurpose},
    Engine as _,
};
use chrono::DateTime;
use async_trait::async_trait;
use oxidauth_kernel::{
    auth::{
        authenticate::{AuthenticateParams, AuthenticateResponse},
        Authenticator,
    },
    authorities::{Authority, AuthorityStrategy},
    error::BoxedError,
    service::Service, jwt::{Jwt, epoch_from_now},
};
use oxidauth_repository::{
    authorities::select_authority_by_strategy::SelectAuthorityByStrategyQuery,
    user_authorities::select_user_authorities_by_authority_id_and_user_identifier::{
        SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
        SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams
    },
    auth::tree::{PermissionSearch, PermissionTreeQuery},
    private_keys::select_most_recent_private_key::{
        SelectMostRecentPrivateKey,
        SelectMostRecentPrivateKeyQuery,
    },
    refresh_tokens::insert_refresh_token::{
        CreateRefreshToken,
        InsertRefreshTokenQuery,
    },
};

const BASE64_ENGINE: GeneralPurpose = general_purpose::STANDARD;

pub struct AuthenticateUseCase<T, U, P, M, R>
where
    T: SelectAuthorityByStrategyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    authority_by_strategy: T,
    user_authority: U,
    permission_tree: P,
    private_keys: M,
    refresh_tokens: R,
}

impl<T, U, P, M, R> AuthenticateUseCase<T, U, P, M, R>
where
    T: SelectAuthorityByStrategyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    pub fn new(
        authority_by_strategy: T,
        user_authority: U,
        permission_tree: P,
        private_keys: M,
        refresh_tokens: R,
    ) -> Self {
        Self {
            authority_by_strategy,
            user_authority,
            permission_tree,
            private_keys,
            refresh_tokens,
        }
    }
}

#[async_trait]
impl<'a, T, U, P, M, R> Service<&'a AuthenticateParams>
    for AuthenticateUseCase<T, U, P, M, R>
where
    T: SelectAuthorityByStrategyQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    type Response = AuthenticateResponse;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a AuthenticateParams,
    ) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authority_by_strategy
            .call(&params.into())
            .await?;

        let authenticator =
            build_authenticator(&authority, &params.strategy).await?;

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

        let permissions = self
            .permission_tree
            .call(&PermissionSearch::User(
                user_authority.user_id,
            ))
            .await?
            .permissions;

        let private_key = self
            .private_keys
            .call(&SelectMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_ENGINE.decode(private_key.private_key)?;

        let jwt = Jwt::new()
            .with_subject(user_authority.user_id)
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
            .refresh_tokens
            .call(&CreateRefreshToken {
                user_id: user_authority.user_id,
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

pub async fn build_authenticator(
    authority: &Authority,
    strategy: &AuthorityStrategy,
) -> Result<Box<dyn Authenticator>, BoxedError> {
    use AuthorityStrategy::*;

    match strategy {
        UsernamePassword => strategies::username_password::new(authority).await,
        SingleUseToken => unimplemented!(),
    }
}

pub mod strategies {
    pub mod username_password {
        use argon2::{Argon2, PasswordHash, password_hash::Error as PasswordHashError, PasswordVerifier};
        use async_trait::async_trait;
        use serde::Deserialize;
        use uuid::Uuid;

        use oxidauth_kernel::{
            auth::Authenticator, authorities::Authority, error::BoxedError, user_authorities::UserAuthority,
        };

        pub async fn new(
            authority: &Authority,
        ) -> Result<Box<dyn Authenticator>, BoxedError> {
            let params: AuthorityParams = authority.params.clone().try_into()?;
            let authority_id = authority.id;
            let password_pepper = std::env::var("OXIDAUTH_USERNAME_PASSWORD_PEPPER")?;

            Ok(Box::new(UsernamePassword {
                authority_id,
                params,
                password_pepper,
            }))
        }

        #[derive(Debug)]
        pub struct UsernamePassword {
            authority_id: Uuid,
            params: AuthorityParams,
            password_pepper: String,
        }

        #[derive(Debug, Deserialize)]
        pub struct AuthorityParams {
            password_salt: String,
        }

        impl TryFrom<serde_json::Value> for AuthorityParams {
            type Error = BoxedError;

            fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
                let s: Self = serde_json::from_value(value)?;

                Ok(s)
            }
        }

        #[async_trait]
        impl Authenticator for UsernamePassword {
            async fn user_identifier_from_request(
                &self,
                params: &serde_json::Value,
            ) -> Result<String, BoxedError> {
                let AuthenticateParams { username, .. } = serde_json::from_value(params.clone())?;

                Ok(username)
            }

            async fn authenticate(
                &self,
                authenticate_params: serde_json::Value,
                user_authority: &UserAuthority,
            ) -> Result<(), BoxedError> {
                let authenticate_params: AuthenticateParams = serde_json::from_value(authenticate_params.clone())?;

                let password = format!(
                    "{}:{}:{}",
                    authenticate_params.password, self.params.password_salt, self.password_pepper,
                    );

                let user_authority_params: UserAuthorityParams =
                    user_authority.params.clone().try_into()?;

                verify_password(password, user_authority_params.password_hash)
                    .map_err(|err| err.to_string())?;

                Ok(())
            }
        }

        #[derive(Clone, Deserialize)]
        pub struct AuthenticateParams {
            pub username: String,
            pub password: String,
        }

        #[derive(Clone, Deserialize)]
        pub struct UserAuthorityParams {
            pub password_hash: String,
        }

        impl TryFrom<serde_json::Value> for UserAuthorityParams {
            type Error = BoxedError;

            fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
                let s: Self = serde_json::from_value(value)?;

                Ok(s)
            }

        }

        pub fn verify_password(password: String, password_hash: String) -> Result<bool, PasswordHashError> {
            let password_hash = PasswordHash::new(&password_hash)?;

            Argon2::default().verify_password(&password.into_bytes(), &password_hash)?;

            Ok(true)
        }
    }
}

pub async fn authenticate(
    params: &AuthenticateParams,
) -> Result<AuthenticateResponse, String> {
    //     let authority = authority_by_strategy(db, &request.strategy)
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     let authenticator = authority_factory(&authority, &request.strategy)
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     let user_identifier = authenticator
    //         .user_identifier_from_request(request.params.clone())
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     let user_authority = user_authority_by_user_identifier(db, user_identifier)
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     authenticator
    //         .authenticate(
    //             request.params,
    //             &user_authority,
    //         )
    //         .await
    //         .map_err(|err| err.to_string())?;
    //
    //     let result = jwt_and_refresh_token(
    //         db,
    //         &authority,
    //         user_authority.user_id,
    //     )
    //     .await
    //     .map_err(|err| err.to_string())?;
    //
    //     Ok(result)
    todo!()
}
