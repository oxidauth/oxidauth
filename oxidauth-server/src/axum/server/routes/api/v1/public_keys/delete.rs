use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct PublicKeysDeleteReq {
    pub public_key_id: Uuid,
}

#[derive(Serialize)]
pub struct PublicKeysDeleteRes {}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<PublicKeysDeleteReq>,
) -> impl IntoResponse {
    let public_key = sqlx::query(QUERY)
        .bind(params.public_key_id)
        .execute(&db)
        .await;

    let public_key = public_key
        .map(|_| PublicKeysDeleteRes {})
        .map_err(|error| error.to_string());

    match public_key {
        Ok(payload) => Response::success(payload).json(),
        Err(errors) => Response::fail(errors).json(),
    }
}

pub const QUERY: &str = r#"
    DELETE FROM public_keys
    WHERE id = $1
"#;
