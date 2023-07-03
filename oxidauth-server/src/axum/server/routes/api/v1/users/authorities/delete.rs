use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct UserAuthorityDeletePathReq {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}

#[derive(Serialize)]
pub struct UserAuthorityDeleteRes {}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserAuthorityDeletePathReq>,
) -> impl IntoResponse {
    let result = sqlx::query(QUERY)
        .bind(params.user_id)
        .bind(params.authority_id)
        .execute(&db)
        .await;

    let result = result
        .map(|_| UserAuthorityDeleteRes {})
        .map_err(|error| error.to_string());

    match result {
        Ok(payload) => Response::success(payload).json(),
        Err(errors) => Response::fail(errors).json(),
    }
}

pub const QUERY: &str = r#"
    DELETE FROM user_authorities
    WHERE user_id = $1
    AND authority_id = $2
"#;
