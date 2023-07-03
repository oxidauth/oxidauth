use axum::{extract::Query, response::IntoResponse, Extension};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{types::Json, PgPool};
use uuid::Uuid;

use crate::{authorities::AuthoritySettings, axum::Response};

#[derive(Deserialize)]
pub struct AuthoritiesAllReq {}

#[derive(Serialize)]
pub struct AuthoritiesAllRes {
    authorities: Vec<AuthorityRow>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Query(_params): Query<AuthoritiesAllReq>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, AuthorityRow>(QUERY)
        .fetch_all(&db)
        .await;

    match result {
        Ok(authorities) => Response::success(AuthoritiesAllRes { authorities }).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    SELECT * FROM authorities
"#;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthorityRow {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: String,
    pub strategy: String,
    pub params: Value,
    pub settings: Json<AuthoritySettings>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
