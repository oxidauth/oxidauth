use std::{
    error::Error as StdError,
    fmt::Debug,
    time::{self, Duration},
};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    axum::server::routes::api::v1::{
        authorities::{all::AuthorityRow, by_authority_strategy::authority_by_strategy},
        refresh_tokens::exchange::refresh_token_create,
        users::{
            authorities::{
                all::UserAuthority,
                create::{user_authority_create, UserAuthorityCreate},
            },
            create::{user_create, UserCreateRow},
            permissions_as_tree, PermissionSourceID,
        },
    },
    jwt::{epoch_from_now, Jwt},
};

pub mod strategies;
pub use strategies::*;

pub type Error = Box<dyn StdError + Send + Sync + 'static>;

pub trait Authority: Registrar + Authenticator + Send + Sync {}

impl<T> Authority for T where T: Registrar + Authenticator {}

#[async_trait]
pub trait Registrar: Send + Sync {
    async fn user_authority_from_request(
        &self,
        params: Value,
    ) -> Result<UserAuthorityCreate, Error>;

    async fn register(
        &self,
        register_params: Value,
    ) -> Result<(UserCreateRow, UserAuthorityCreate), Error>;
}

#[async_trait]
pub trait Authenticator: Send + Sync {
    async fn user_identifier_from_request(&self, params: Value) -> Result<String, Error>;

    async fn authenticate(
        &self,
        authenticate_params: Value,
        user_authority: &UserAuthority,
    ) -> Result<(), Error>;
}

#[derive(Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "snake_case")]
pub enum AuthorityStrategy {
    UsernamePassword,
}

impl Debug for AuthorityStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "username_password")
    }
}

impl From<&str> for AuthorityStrategy {
    fn from(_string: &str) -> Self {
        Self::UsernamePassword
    }
}

pub async fn register(db: &mut PgConnection, request: Request) -> Result<Response, Error> {
    let authority = authority_by_strategy(db, &request.strategy)
        .await
        .map_err(|err| err.to_string())?;

    let registrar = authority_factory(&authority, &request.strategy).await?;

    let (user, user_authority) = registrar.register(request.params).await?;

    let user = user_create(db, user).await?;

    let _ = user_authority_create(db, user.id, user_authority).await?;

    // add default roles and permissions

    let result = jwt_and_refresh_token(db, &authority, user.id).await?;

    Ok(result)
}

pub async fn authenticate(db: &mut PgConnection, request: Request) -> Result<Response, String> {
    let authority = authority_by_strategy(db, &request.strategy)
        .await
        .map_err(|err| err.to_string())?;

    let authenticator = authority_factory(&authority, &request.strategy)
        .await
        .map_err(|err| err.to_string())?;

    let user_identifier = authenticator
        .user_identifier_from_request(request.params.clone())
        .await
        .map_err(|err| err.to_string())?;

    let user_authority = user_authority_by_user_identifier(db, user_identifier)
        .await
        .map_err(|err| err.to_string())?;

    authenticator
        .authenticate(request.params, &user_authority)
        .await
        .map_err(|err| err.to_string())?;

    let result = jwt_and_refresh_token(db, &authority, user_authority.user_id)
        .await
        .map_err(|err| err.to_string())?;

    Ok(result)
}

pub async fn jwt_and_refresh_token(
    db: &mut PgConnection,
    authority: &AuthorityRow,
    user_id: Uuid,
) -> Result<Response, Error> {
    let permissions = permissions_as_tree(db, PermissionSourceID::User(user_id))
        .await?
        .permissions;

    let private_key = private_key_most_recent(db).await?;

    let private_key = base64::decode(private_key.private_key)?;

    let jwt = Jwt::new()
        .with_subject(user_id)
        .with_issuer("oxidauth".to_string())
        .with_expires_in(authority.settings.jwt_ttl)
        .with_entitlements(permissions)
        .with_not_before_from(Duration::from_secs(0))
        .build()
        .map_err(|err| format!("unable to build jwt: {}", err))?
        .encode(&private_key)
        .map_err(|err| format!("unable to encode jwt: {}", err))?;

    let refresh_token_exp_at = epoch_from_now(authority.settings.refresh_token_ttl)?;

    let refresh_token_exp_at = NaiveDateTime::from_timestamp(refresh_token_exp_at as i64, 0);

    let refresh_token =
        refresh_token_create(db, user_id, authority.id, refresh_token_exp_at).await?;

    Ok(Response {
        jwt,
        refresh_token: refresh_token.id,
    })
}

pub async fn authority_factory(
    authority: &AuthorityRow,
    strategy: &AuthorityStrategy,
) -> Result<Box<dyn Authority>, Error> {
    use AuthorityStrategy::*;

    match strategy {
        UsernamePassword => username_password::new(authority),
    }
    .await
}

pub async fn private_keys_all(db: &mut PgConnection) -> Result<Vec<PrivateKeyRow>, sqlx::Error> {
    let result = sqlx::query_as::<_, PrivateKeyRow>(PRIVATE_KEYS_ALL_QUERY)
        .fetch_all(db)
        .await?;

    Ok(result)
}

const PRIVATE_KEYS_ALL_QUERY: &str = r#"
    SELECT
        id,
        private_key,
        created_at,
        updated_at
    FROM public_keys
    ORDER BY created_at DESC
"#;

pub async fn private_key_most_recent(db: &mut PgConnection) -> Result<PrivateKeyRow, sqlx::Error> {
    let result = sqlx::query_as::<_, PrivateKeyRow>(PRIVATE_KEY_QUERY)
        .fetch_one(db)
        .await?;

    Ok(result)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PrivateKeyRow {
    pub id: Uuid,
    pub private_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const PRIVATE_KEY_QUERY: &str = r#"
    SELECT
        id,
        private_key,
        created_at,
        updated_at
    FROM public_keys
    ORDER BY created_at DESC
    LIMIT 1
"#;

const USER_AUTHORITY_BY_IDENTIFIER_QUERY: &str = r#"
    SELECT *
    FROM user_authorities
    WHERE user_identifier = $1
"#;

pub async fn user_authority_by_user_identifier(
    db: &mut PgConnection,
    identifier: String,
) -> Result<UserAuthority, Error> {
    let result = sqlx::query_as::<_, UserAuthority>(USER_AUTHORITY_BY_IDENTIFIER_QUERY)
        .bind(identifier)
        .fetch_one(db)
        .await?;

    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub strategy: AuthorityStrategy,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub jwt: String,
    pub refresh_token: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoritySettings {
    pub jwt_ttl: time::Duration,
    pub refresh_token_ttl: time::Duration,
}
