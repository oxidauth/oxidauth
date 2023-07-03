use axum::{extract::Json, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    authorities::{AuthoritySettings, AuthorityStrategy},
    axum::{server::routes::api::v1::public_keys::create::public_key_create, Response},
};

use super::all::AuthorityRow;

#[derive(Deserialize)]
pub struct AuthorityCreateReq {
    authority: AuthorityCreateRow,
}
#[derive(Serialize)]
pub struct AuthorityCreateRes {
    authority: AuthorityRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Json(req): Json<AuthorityCreateReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let mut authority = req.authority;

    if authority.client_key.is_none() {
        authority.client_key.replace(Uuid::new_v4());
    }

    if authority.status.is_none() {
        authority.status.replace("active".into());
    }

    match public_key_create(&mut db).await {
        Ok(_) => {}
        Err(error) => return Response::fail(error.to_string()).json(),
    }

    let result = sqlx::query_as::<_, AuthorityRow>(QUERY)
        .bind(authority.name)
        .bind(authority.client_key)
        .bind(authority.status)
        .bind(authority.strategy)
        .bind(authority.params)
        .bind(authority.settings)
        .fetch_one(&mut db)
        .await;

    match result {
        Ok(authority) => Response::success(AuthorityCreateRes { authority }).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    INSERT INTO authorities (
        name, client_key,
        status, strategy,
        params, settings
    )
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING *
"#;

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct AuthorityCreateRow {
    pub name: String,
    pub client_key: Option<Uuid>,
    pub status: Option<String>,
    pub strategy: AuthorityStrategy,
    pub params: Value,
    pub settings: sqlx::types::Json<AuthoritySettings>,
}
