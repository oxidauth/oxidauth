use axum::{extract::Json, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool};

use crate::axum::Response;

use super::{
    all::UserRow,
    authorities::{
        all::UserAuthority,
        create::{user_authority_create, UserAuthorityCreate},
    },
};

#[derive(Deserialize)]
pub struct UserCreateReq {
    user: UserCreateRow,
    user_authorities: Option<Vec<UserAuthorityCreate>>,
}

#[derive(Serialize)]
pub struct UserCreateRes {
    user: UserRow,
    user_authorities: Option<Vec<UserAuthority>>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Json(body): Json<UserCreateReq>,
) -> impl IntoResponse {
    let mut user = body.user;

    if user.status.is_none() {
        user.status.replace("active".into());
    }

    if user.kind.is_none() {
        user.kind.replace("human".into());
    }

    if user.profile.is_none() {
        user.profile.replace(json!({}));
    }

    match create_user_and_user_authorities(&db, user, body.user_authorities).await {
        Ok((user, user_authorities)) => Response::success(UserCreateRes {
            user,
            user_authorities,
        })
        .json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

pub async fn create_user_and_user_authorities(
    db: &PgPool,
    user: UserCreateRow,
    user_authorities: Option<Vec<UserAuthorityCreate>>,
) -> Result<(UserRow, Option<Vec<UserAuthority>>), sqlx::Error> {
    let mut db = db.begin().await?;

    let user = user_create(&mut db, user).await?;

    let user_authorities = match user_authorities {
        Some(user_authorities) => {
            let mut list = Vec::new();

            for user_authority in user_authorities.into_iter() {
                let created = user_authority_create(&mut db, user.id, user_authority).await?;
                list.push(created);
            }

            Some(list)
        }
        None => None,
    };

    db.commit().await?;

    Ok((user, user_authorities))
}

const QUERY: &str = r#"
    INSERT INTO users (
        username, email,
        first_name, last_name,
        status, kind, profile
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    RETURNING *
"#;

pub async fn user_create(
    db: &mut PgConnection,
    user: UserCreateRow,
) -> Result<UserRow, sqlx::Error> {
    let result = sqlx::query_as::<_, UserRow>(QUERY)
        .bind(user.username)
        .bind(user.email)
        .bind(user.first_name)
        .bind(user.last_name)
        .bind(user.status)
        .bind(user.kind)
        .bind(user.profile)
        .fetch_one(db)
        .await?;

    Ok(result)
}

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct UserCreateRow {
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<String>,
    pub kind: Option<String>,
    pub profile: Option<Value>,
}
