use std::string::FromUtf8Error;

use axum::{extract::Path, response::IntoResponse, Extension};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct PublicKeysAllReq {}

#[derive(Serialize)]
pub struct PublicKeysAllRes {
    pub public_keys: Vec<PublicKey>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(_params): Path<PublicKeysAllReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let public_key_rows = public_keys_all(&mut db).await;

    let public_key_rows = match public_key_rows {
        Ok(public_keys) => public_keys,
        Err(error) => {
            return Response::fail(format!("error fetching public keys: {}", error)).json()
        }
    };

    let mut public_keys: Vec<PublicKey> = vec![];

    for public_key in public_key_rows.into_iter() {
        let public_key: PublicKey = match public_key.try_into() {
            Ok(public_key) => public_key,
            Err(error) => {
                return Response::fail(format!(
                    "error converting public key row into public key: {}",
                    error
                ))
                .json()
            }
        };

        public_keys.push(public_key);
    }

    Response::success(PublicKeysAllRes { public_keys }).json()
}

pub async fn public_keys_all(db: &mut PgConnection) -> Result<Vec<PublicKeyRow>, sqlx::Error> {
    let public_keys = sqlx::query_as::<_, PublicKeyRow>(QUERY)
        .fetch_all(db)
        .await?;

    Ok(public_keys)
}

pub const QUERY: &str = r#"
    SELECT
        id,
        public_key,
        created_at,
        updated_at
    FROM public_keys
"#;

#[derive(Serialize, sqlx::FromRow)]
pub struct PublicKey {
    pub id: Uuid,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct PublicKeyRow {
    pub id: Uuid,
    pub public_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<PublicKeyRow> for PublicKey {
    type Error = FromUtf8Error;

    fn try_from(from: PublicKeyRow) -> Result<Self, Self::Error> {
        let public_key = String::from_utf8(from.public_key)?;

        Ok(Self {
            id: from.id,
            public_key,
            created_at: from.created_at,
            updated_at: from.updated_at,
        })
    }
}
