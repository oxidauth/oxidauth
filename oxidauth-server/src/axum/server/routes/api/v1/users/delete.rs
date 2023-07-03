use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct UserDeleteByIDReq {
    user_id: Uuid,
}

#[derive(Serialize)]
pub struct UserDeleteByIDRes {}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserDeleteByIDReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = user_delete(&mut db, params.user_id).await;

    match result {
        Ok(_) => Response::success(UserDeleteByIDRes {}).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

pub async fn user_delete(db: &mut PgConnection, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(DELETE_USER_QUERY)
        .bind(user_id)
        .execute(db)
        .await?;

    Ok(())
}

const DELETE_USER_QUERY: &'static str = r#"
    DELETE FROM users
    WHERE id = $1
"#;
