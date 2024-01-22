use async_trait::async_trait;
use base64::{
    engine::{general_purpose, GeneralPurpose},
    Engine as _,
};
use chrono::DateTime;
use oxidauth_kernel::{
    auth::{
        register::{RegisterParams, RegisterResponse},
        Registrar,
    },
    authorities::{Authority, AuthorityStrategy},
    error::BoxedError,
    jwt::{epoch_from_now, Jwt},
    service::Service,
};
use oxidauth_repository::{
    auth::tree::{PermissionSearch, PermissionTreeQuery},
    authorities::select_authority_by_strategy::SelectAuthorityByStrategyQuery,
    private_keys::select_most_recent_private_key::{
        SelectMostRecentPrivateKey, SelectMostRecentPrivateKeyQuery,
    },
    refresh_tokens::insert_refresh_token::{
        CreateRefreshToken, InsertRefreshTokenQuery,
    },
    user_authorities::insert_user_authority::InsertUserAuthorityQuery,
    users::insert_user::InsertUserQuery,
};
use std::time::Duration;

const BASE64_ENGINE: GeneralPurpose = general_purpose::STANDARD;

pub struct RegisterUseCase<T, U, A, P, M, R>
where
    T: SelectAuthorityByStrategyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    authority_by_strategy: T,
    users: U,
    user_authorities: A,
    permission_tree: P,
    private_keys: M,
    refresh_tokens: R,
}

impl<T, U, A, P, M, R> RegisterUseCase<T, U, A, P, M, R>
where
    T: SelectAuthorityByStrategyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    pub fn new(
        authority_by_strategy: T,
        users: U,
        user_authorities: A,
        permission_tree: P,
        private_keys: M,
        refresh_tokens: R,
    ) -> Self {
        Self {
            authority_by_strategy,
            users,
            user_authorities,
            permission_tree,
            private_keys,
            refresh_tokens,
        }
    }
}

#[async_trait]
impl<'a, T, U, A, P, M, R> Service<&'a RegisterParams>
    for RegisterUseCase<T, U, A, P, M, R>
where
    T: SelectAuthorityByStrategyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    type Response = RegisterResponse;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a RegisterParams,
    ) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authority_by_strategy
            .call(&params.into())
            .await?;

        let registrar = build_registrar(&authority, &params.strategy).await?;

        let (user, user_authority) = registrar
            .register(params.params.clone())
            .await?;

        let user = self.users.call(&user).await?;

        self.user_authorities
            .call(&user_authority)
            .await?;

        // add default roles and permissions

        let permissions = self
            .permission_tree
            .call(&PermissionSearch::User(
                user.id,
            ))
            .await?
            .permissions;

        let private_key = self
            .private_keys
            .call(&SelectMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_ENGINE.decode(private_key.private_key)?;

        let jwt = Jwt::new()
            .with_subject(user.id)
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
                user_id: user.id,
                authority_id: authority.id,
                expires_at: refresh_token_exp_at,
            })
            .await?;

        Ok(RegisterResponse {
            jwt,
            refresh_token: refresh_token.id,
        })
    }
}

pub async fn build_registrar(
    authority: &Authority,
    strategy: &AuthorityStrategy,
) -> Result<Box<dyn Registrar>, BoxedError> {
    use AuthorityStrategy::*;

    match strategy {
        UsernamePassword => strategies::username_password::new(authority).await,
        SingleUseToken => unimplemented!(),
    }
}

pub mod strategies {
    pub mod username_password {
        use argon2::{
            password_hash::{Error as PasswordHashError, SaltString},
            Argon2, PasswordHasher,
        };
        use async_trait::async_trait;
        use rand_core::OsRng;
        use serde::{Deserialize, Serialize};
        use serde_json::Value;
        use uuid::Uuid;

        use oxidauth_kernel::{
            auth::Registrar,
            authorities::Authority,
            error::BoxedError,
            user_authorities::create_user_authority::CreateUserAuthority,
            users::{create_user::CreateUser, UserKind, UserStatus},
        };

        pub async fn new(
            authority: &Authority,
        ) -> Result<Box<dyn Registrar>, BoxedError> {
            let params: AuthorityParams = authority
                .params
                .clone()
                .try_into()?;
            let authority_id = authority.id;
            let password_pepper =
                std::env::var("OXIDAUTH_USERNAME_PASSWORD_PEPPER")?;

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
        impl Registrar for UsernamePassword {
            async fn register(
                &self,
                register_params: serde_json::Value,
            ) -> Result<
                (
                    CreateUser,
                    CreateUserAuthority,
                ),
                BoxedError,
            > {
                let register_params: RegisterParams = register_params
                    .clone()
                    .try_into()?;

                if register_params.password
                    != register_params.password_confirmation
                {
                    return Err(
                        "password and password confirmation do not match"
                            .into(),
                    );
                }

                let user: CreateUser = register_params.clone().into();

                let password = format!(
                    "{}:{}:{}",
                    register_params.password,
                    self.params.password_salt,
                    self.password_pepper,
                );

                let password_hash =
                    hash_password(password).map_err(|err| err.to_string())?;
                let params = UserAuthorityParams { password_hash };

                let params = serde_json::to_value(params)?;

                let user_authority = CreateUserAuthority {
                    user_id: user.id,
                    authority_id: self.authority_id,
                    user_identifier: user.username.clone(),
                    params,
                };

                Ok((user, user_authority))
            }
        }

        #[derive(Clone, Deserialize)]
        pub struct RegisterParams {
            pub username: String,
            pub password: String,
            pub password_confirmation: String,
            pub email: Option<String>,
            pub first_name: Option<String>,
            pub last_name: Option<String>,
        }

        impl TryFrom<Value> for RegisterParams {
            type Error = BoxedError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                let s: Self = serde_json::from_value(value)?;

                Ok(s)
            }
        }

        impl From<RegisterParams> for CreateUser {
            fn from(params: RegisterParams) -> Self {
                let RegisterParams {
                    username,
                    email,
                    first_name,
                    last_name,
                    ..
                } = params.clone();
                let user_id = Uuid::new_v4();
                let status = Some(UserStatus::Enabled);
                let kind = Some(UserKind::Human);

                Self {
                    id: Some(user_id),
                    username,
                    email,
                    first_name,
                    last_name,
                    status,
                    kind,
                    profile: Some(Value::default()),
                }
            }
        }

        #[derive(Clone, Serialize, Deserialize)]
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

        pub fn hash_password(
            password: String,
        ) -> Result<String, PasswordHashError> {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();

            let password_hash = argon2
                .hash_password(&password.into_bytes(), &salt)?
                .to_string();

            Ok(password_hash)
        }
    }
}
