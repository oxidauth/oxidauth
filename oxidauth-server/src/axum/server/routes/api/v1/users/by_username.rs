use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};

use crate::axum::Response;

use super::all::UserRow;

#[derive(Deserialize)]
pub struct UserByUsernameReq {
    username: String,
}

#[derive(Serialize)]
pub struct UserByUsernameRes {
    user: UserRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserByUsernameReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = by_username(&mut db, &params.username).await;

    match result {
        Ok(user) => Response::success(UserByUsernameRes { user }).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &'static str = r#"
    SELECT *
    FROM users
    WHERE username = $1
"#;

pub async fn by_username(db: &mut PgConnection, username: &String) -> Result<UserRow, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(QUERY)
        .bind(username)
        .fetch_one(db)
        .await
}
