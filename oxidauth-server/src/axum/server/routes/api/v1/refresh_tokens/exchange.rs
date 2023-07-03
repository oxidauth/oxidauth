use std::{error::Error as StdError, fmt::Display, time::SystemTime};

use axum::{response::IntoResponse, Extension, Json};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    authorities::{jwt_and_refresh_token, Response as AuthResponse},
    axum::{
        server::routes::api::v1::{
            authorities::by_id::authority_by_id,
            users::authorities::by_id::user_authority_by_user_id_and_authority_id,
        },
        Response,
    },
    jwt::epoch_from_time,
};

#[derive(Deserialize)]
pub struct RefreshTokenExchangeReq {
    refresh_token: Uuid,
}

#[derive(Serialize)]
pub struct RefreshTokenExchangeRes {
    pub refresh_token: Uuid,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Json(body): Json<RefreshTokenExchangeReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let response = match exchange(&mut db, body.refresh_token).await {
        Ok(response) => response,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    Response::success(response).json()
}

pub async fn exchange(db: &mut PgConnection, refresh_token: Uuid) -> Result<AuthResponse, Error> {
    let RefreshTokenRow {
        user_id,
        authority_id,
        expires_at,
        ..
    } = refresh_token_by_id(db, refresh_token)
        .await
        .map_err(|err| Error::SQLXError(err.to_string()))?;

    let now = epoch_from_time(SystemTime::now()).map_err(|_| Error::RefreshTokenExpAt)?;

    if expires_at.timestamp() < now as i64 {
        return Err(Error::RefreshTokenExpAt);
    }

    let authority_id = user_authority_by_user_id_and_authority_id(db, user_id, authority_id)
        .await
        .map_err(|err| Error::SQLXError(err.to_string()))?
        .authority_id;

    let authority = authority_by_id(db, authority_id)
        .await
        .map_err(|err| Error::SQLXError(err.to_string()))?;

    let response = jwt_and_refresh_token(db, &authority, user_id)
        .await
        .map_err(|err| Error::SQLXError(err.to_string()))?;

    Ok(response)
}

#[derive(Debug)]
pub enum Error {
    SQLXError(String),
    MismatchedRefreshToken,
    RefreshTokenExpAt,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SQLXError(error) => write!(f, "database error: {}", error),
            Self::MismatchedRefreshToken => write!(f, "bad refresh token"),
            Self::RefreshTokenExpAt => write!(f, "unable to make refresh token ttl"),
        }
    }
}

impl StdError for Error {}

const BY_ID_QUERY: &str = r#"
    SELECT *
    FROM refresh_tokens
    WHERE id = $1
"#;

pub async fn refresh_token_by_id(
    db: &mut PgConnection,
    refresh_token: Uuid,
) -> Result<RefreshTokenRow, sqlx::Error> {
    let result = sqlx::query_as::<_, RefreshTokenRow>(BY_ID_QUERY)
        .bind(refresh_token)
        .fetch_one(db)
        .await?;

    Ok(result)
}

const QUERY_CREATE: &str = r#"
    INSERT INTO refresh_tokens
    (user_id, authority_id, expires_at)
    VALUES ($1, $2, $3)
    RETURNING *
"#;

pub async fn refresh_token_create(
    db: &mut PgConnection,
    user_id: Uuid,
    authority_id: Uuid,
    expires_at: NaiveDateTime,
) -> Result<RefreshTokenRow, sqlx::Error> {
    let result = sqlx::query_as::<_, RefreshTokenRow>(QUERY_CREATE)
        .bind(user_id)
        .bind(authority_id)
        .bind(expires_at)
        .fetch_one(db)
        .await?;

    Ok(result)
}

#[derive(sqlx::FromRow)]
pub struct RefreshTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
