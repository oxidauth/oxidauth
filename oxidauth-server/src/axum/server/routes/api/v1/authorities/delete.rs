use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct AuthorityDeleteByIDReq {
    authority_id: Uuid,
}

#[derive(Serialize)]
pub struct AuthorityDeleteByIDRes {}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<AuthorityDeleteByIDReq>,
) -> impl IntoResponse {
    let result = sqlx::query(QUERY)
        .bind(params.authority_id)
        .execute(&db)
        .await;

    match result {
        Ok(_) => Response::success(AuthorityDeleteByIDRes {}).json(),
        Err(error) => {
            Response::fail(format!("error deleting authority: {}", error.to_string())).json()
        }
    }
}

pub const QUERY: &str = r#"
    DELETE FROM authorities
    WHERE id = $1
"#;
