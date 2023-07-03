use axum::{
    extract::{Json, Path},
    response::IntoResponse,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

use super::{all::AuthorityRow, by_id};

#[derive(Deserialize)]
pub struct AuthorityUpdatePathReq {
    authority_id: Uuid,
}

#[derive(Deserialize)]
pub struct AuthorityUpdateBodyReq {
    authority: AuthorityUpdateRow,
}

#[derive(Serialize)]
pub struct AuthorityUpdateRes {
    authority: AuthorityRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<AuthorityUpdatePathReq>,
    Json(body): Json<AuthorityUpdateBodyReq>,
) -> impl IntoResponse {
    let mut authority = body.authority;

    authority.id = Some(params.authority_id);

    let current = sqlx::query_as::<_, AuthorityRow>(by_id::QUERY)
        .bind(params.authority_id)
        .fetch_one(&db)
        .await;

    let current = match current {
        Ok(authority) => authority,
        Err(_error) => return Response::fail(String::from("authority not found")).json(),
    };

    if authority.client_key.is_none() {
        authority.client_key.replace(Uuid::new_v4());
    }

    if authority.status.is_none() {
        authority.status = Some(current.status);
    }

    let result = sqlx::query_as::<_, AuthorityRow>(QUERY)
        .bind(authority.id)
        .bind(authority.name)
        .bind(authority.client_key)
        .bind(authority.status)
        .bind(authority.strategy)
        .bind(authority.params)
        .bind(authority.settings)
        .fetch_one(&db)
        .await;

    match result {
        Ok(authority) => Response::success(AuthorityUpdateRes { authority }).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    UPDATE authorities
    SET
        name = $2,
        client_key = $3,
        status = $4,
        strategy = $5,
        params = $6,
        settings = $7
    WHERE id = $1
    RETURNING *
"#;

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct AuthorityUpdateRow {
    pub id: Option<Uuid>,
    pub name: String,
    pub client_key: Option<Uuid>,
    pub status: Option<String>,
    pub strategy: String,
    pub params: Value,
    pub settings: Value,
}
