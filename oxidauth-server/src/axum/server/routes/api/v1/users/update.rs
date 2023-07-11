use axum::{
    extract::{Json, Path},
    response::IntoResponse,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::Response;

use super::{all::UserRow, by_id};

#[derive(Deserialize)]
pub struct UserUpdatePathReq {
    user_id: Uuid,
}

#[derive(Deserialize)]
pub struct UserUpdateBodyReq {
    user: UserUpdateRow,
}

#[derive(Serialize)]
pub struct UserUpdateRes {
    user: UserRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserUpdatePathReq>,
    Json(request): Json<UserUpdateBodyReq>,
) -> impl IntoResponse {
    let mut updates = request.user;

    updates.id = Some(params.user_id);

    let result = user_update(db, updates).await;

    match result {
        Ok(user) => Response::success(UserUpdateRes { user }).json(),
        Err(error) => Response::fail(format!("error updating user: {}", error)).json(),
    }
}

pub async fn user_update(db: PgPool, mut updates: UserUpdateRow) -> Result<UserRow, sqlx::Error> {
    let mut db = db.begin().await?;

    let current = sqlx::query_as::<_, UserRow>(by_id::QUERY)
        .bind(updates.id)
        .fetch_one(&mut db)
        .await?;

    if updates.email.is_none() {
        updates.email = current.email;
    }

    if updates.status.is_none() {
        updates.status = Some(current.status);
    }

    if updates.profile.is_none() {
        updates.profile = Some(current.profile);
    }

    let result = sqlx::query_as::<_, UserRow>(QUERY)
        .bind(updates.id)
        .bind(updates.email)
        .bind(updates.first_name)
        .bind(updates.last_name)
        .bind(updates.status)
        .bind(updates.profile)
        .fetch_one(&mut db)
        .await?;

    Ok(result)
}

const QUERY: &str = r#"
    UPDATE users
    SET
        email = $2,
        first_name = $3,
        last_name = $4,
        status = $5,
        profile = $6
    WHERE id = $1
    RETURNING *;
"#;

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct UserUpdateRow {
    pub id: Option<Uuid>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<String>,
    pub profile: Option<Value>,
}
