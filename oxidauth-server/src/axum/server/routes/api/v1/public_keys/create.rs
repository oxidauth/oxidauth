use std::error::Error as StdError;

use axum::{response::IntoResponse, Extension};
use serde::Serialize;
use sqlx::{PgConnection, PgPool};

use crate::{
    axum::Response,
    rsa::{generate, Base64KeyPair},
};

use super::all::{PublicKey, PublicKeyRow};

type Error = Box<dyn StdError>;

#[derive(Serialize)]
pub struct PublicKeyCreateRes {
    pub public_key: PublicKey,
}

pub async fn handler(Extension(db): Extension<PgPool>) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let public_key = match public_key_create(&mut db).await {
        Ok(public_key) => public_key,
        Err(errors) => return Response::fail(errors.to_string()).json(),
    };

    Response::success(PublicKeyCreateRes { public_key }).json()
}

pub async fn public_key_create(db: &mut PgConnection) -> Result<PublicKey, Error> {
    let Base64KeyPair { public, private } = generate()?.base64_encode();

    let public_key = sqlx::query_as::<_, PublicKeyRow>(QUERY)
        .bind(public)
        .bind(private)
        .fetch_one(db)
        .await?
        .try_into()?;

    Ok(public_key)
}

pub const QUERY: &str = r#"
    INSERT INTO public_keys
    (public_key, private_key)
    VALUES($1, $2)
    RETURNING id, public_key, created_at, updated_at
"#;
