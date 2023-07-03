use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

use super::all::{PublicKey, PublicKeyRow};

#[derive(Deserialize)]
pub struct PublicKeysAllReq {
    pub public_key_id: Uuid,
}

#[derive(Serialize)]
pub struct PublicKeysAllRes {
    pub public_key: PublicKey,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<PublicKeysAllReq>,
) -> impl IntoResponse {
    let public_key = sqlx::query_as::<_, PublicKeyRow>(QUERY)
        .bind(params.public_key_id)
        .fetch_one(&db)
        .await;

    let public_key_row = match public_key {
        Ok(public_key) => public_key,
        Err(error) => {
            return Response::fail(format!("error fetching public keys: {}", error)).json()
        }
    };

    let public_key: PublicKey = match public_key_row.try_into() {
        Ok(public_key) => public_key,
        Err(error) => {
            return Response::fail(format!(
                "error converting public key row into public key: {}",
                error
            ))
            .json()
        }
    };

    Response::success(PublicKeysAllRes { public_key }).json()
}

pub const QUERY: &str = r#"
    SELECT
        id,
        public_key,
        created_at,
        updated_at
    FROM public_keys
    WHERE id = $1
"#;
