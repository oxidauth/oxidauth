use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::Response;

use super::all::UserRow;

#[derive(Deserialize)]
pub struct UserByIDReq {
    user_id: Uuid,
}

#[derive(Serialize)]
pub struct UserByIDRes {
    user: UserRow,
}

#[axum_macros::debug_handler]
pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserByIDReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = user_by_id(&mut db, params.user_id).await;

    match result {
        Ok(user) => Response::success(UserByIDRes { user }).json(),
        Err(error) => Response::fail(format!("user not found: {}", error.to_string())).json(),
    }
}

pub const QUERY: &'static str = r#"
    SELECT *
    FROM users
    WHERE id = $1
"#;

pub async fn user_by_id(db: &mut PgConnection, user_id: Uuid) -> Result<UserRow, sqlx::Error> {
    let result = sqlx::query_as::<_, UserRow>(QUERY)
        .bind(user_id)
        .fetch_one(db)
        .await?;

    Ok(result)
}
